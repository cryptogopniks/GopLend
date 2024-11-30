import { getSigner } from "../account/signer";
import { l, wait, Request, li } from "../../common/utils";
import { readFile } from "fs/promises";
import { getChainOptionById } from "../../common/config/config-utils";
import { ChainConfig } from "../../common/interfaces";
import { getFloorPrices } from "./sg-api";
import { COLLECTION } from "./constants";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import {
  CollectionPriceResponse,
  getOffersExtendedDown,
  getOffersExtendedUp,
  splitOffers,
} from "./math";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
  getStargazeCollections,
} from "./utils";

const DELAY = 6_000;
// TODO: move in contract MAX_OFFERS and MAX_INACTIVE_OFFERS per collection
const MAX_OFFERS = 15;
const MAX_INACTIVE_OFFERS = 5;
const FLOOR_PRICE_BOUNDARY = {
  UPPER: 0.93,
  LOWER: 0.8,
};
const ALLOWED_OFFERS_BOUNDARY = {
  UPPER: 0.93, // 0.8325,
  LOWER: 0.8,
};
const PAGINATION = {
  COLLECTIONS: 50,
};

async function main(isInfiniteLoop: boolean) {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    NAME,
    PREFIX,
    OPTION: {
      RPC_LIST: [RPC],
      DENOM,
      GAS_PRICE_AMOUNT,
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
  const testWallets = await getWallets(TYPE);
  const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

  const { marketMaker, lending } = await getCwQueryHelpers(chainId, RPC);
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);
  const req = new Request();

  const mainnetCollectionEntries = Object.entries(COLLECTION?.MAINNET || {});
  const collectionEntries =
    TYPE === "main"
      ? mainnetCollectionEntries
      : Object.entries(COLLECTION?.TESTNET || {});

  const collectionList = (
    await lending.pQueryCollectionList(PAGINATION.COLLECTIONS)
  ).map((x) => x.address);

  const stargazeCollectionList = getStargazeCollections(
    collectionList,
    mainnetCollectionEntries,
    collectionEntries
  );

  const collectionAndFloorPriceList = await getFloorPrices(
    stargazeCollectionList
  );

  // [collection, CollectionPriceResponse][]
  let collectionPriceResponseList: [string, CollectionPriceResponse][] =
    collectionAndFloorPriceList.map((x, i) => {
      const collection = collectionList[i];
      const collectionPriceResponse: CollectionPriceResponse = {
        twapSampleList: [],
        floorPriceEma: 0,
        floorPrice: x.price * 1e6,
        twapOfferList: [],
        offerEma: 0,
        collectionPrice: 0,
      };

      return [collection, collectionPriceResponse];
    });

  // loop
  while (true) {
    try {
      // TODO: query collectionPriceResponseList from DB
      // collectionPriceResponseList = await queryCollectionPriceResponseList();

      for (const [collection, { floorPrice }] of collectionPriceResponseList) {
        const liquidity = await marketMaker.cwQueryLiquidity(collection);
        const amountUndistributed = Number(liquidity.amount_undistributed);
        const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
        const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
        const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
          lowerFloorPrice,
          upperFloorPrice,
          liquidity.price_list
        );

        // add initial offers or extend offers up
        if (
          !regularOffers.length ||
          tooSmallOffers.length >= MAX_INACTIVE_OFFERS
        ) {
          const { fromToPriceList, isNoLiquidity, isRequiredMoreLiquidity } =
            getOffersExtendedUp(
              ALLOWED_OFFERS_BOUNDARY.UPPER,
              amountUndistributed,
              floorPrice,
              lowerFloorPrice,
              regularOffers,
              tooSmallOffers,
              MAX_OFFERS
            );

          if (isNoLiquidity) {
            l(`⚠️⚠️⚠️ Warning: No liquidity for collection ${collection}`);
            continue;
          }

          if (isRequiredMoreLiquidity) {
            l(`⚠️⚠️⚠️ Warning: Add liquidity for collection ${collection}`);
          }

          await h.marketMaker.cwUpdateOffers(
            collection,
            fromToPriceList,
            gasPrice
          );
          continue;
        }

        // extend offers down
        if (tooBigOffers.length >= MAX_INACTIVE_OFFERS) {
          const { fromToPriceList, isNoLiquidity } = getOffersExtendedDown(
            ALLOWED_OFFERS_BOUNDARY.LOWER,
            amountUndistributed,
            floorPrice,
            upperFloorPrice,
            regularOffers,
            tooBigOffers
          );

          if (isNoLiquidity) {
            l(`⚠️⚠️⚠️ Warning: No liquidity for collection ${collection}`);
            continue;
          }

          await h.marketMaker.cwUpdateOffers(
            collection,
            fromToPriceList,
            gasPrice
          );
        }
      }
    } catch (error) {
      l(error, "\n");
    }
    await wait(DELAY);

    if (!isInfiniteLoop) break;
  }
}

// TODO: set true for production
main(false);
