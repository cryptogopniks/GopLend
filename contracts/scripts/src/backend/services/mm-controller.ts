import { getSigner } from "../account/signer";
import { l, wait, Request, li, floor } from "../../common/utils";
import { readFile } from "fs/promises";
import { getChainOptionById } from "../../common/config/config-utils";
import { ChainConfig } from "../../common/interfaces";
import { getFloorPrices } from "./sg-api";
import { COLLECTION } from "./constants";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import { calcOffers, CollectionPriceResponse, splitOffers } from "./math";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
  getStargazeCollections,
  getLocalBlockTime,
} from "./utils";

const DELAY = 6_000;
const OFFERS_UPDATE_PERIOD = 24 * 3_600;
const LIQUIDITY_CONSTANT = 500;
const MIN_OFFERS = 20;
const MAX_OFFERS = 100;
const MAX_INACTIVE_OFFERS_RATE = 0.25;
const FLOOR_PRICE_BOUNDARY = {
  UPPER: 0.93,
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

  const updateTime = getLocalBlockTime() + OFFERS_UPDATE_PERIOD;
  let collectionAndDateList: [string, number][] = collectionList.map(
    (collection) => [collection, updateTime]
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
        const targetOffersAmount = Math.min(
          MIN_OFFERS +
            floor(
              (2 * LIQUIDITY_CONSTANT) / (lowerFloorPrice + upperFloorPrice)
            ),
          MAX_OFFERS
        );

        const { regularOffers } = splitOffers(
          lowerFloorPrice,
          upperFloorPrice,
          liquidity.price_list
        );

        const blockTime = getLocalBlockTime();
        const updateTime =
          collectionAndDateList.find(
            ([currentCollection, _]) => currentCollection === collection
          )?.[1] || 0;

        if (
          regularOffers.length <=
            targetOffersAmount * (1 - MAX_INACTIVE_OFFERS_RATE) ||
          blockTime >= updateTime
        ) {
          const offers = calcOffers(
            lowerFloorPrice,
            upperFloorPrice,
            liquidity.price_list,
            amountUndistributed,
            MAX_OFFERS
          );

          if (
            !offers.length ||
            JSON.stringify(regularOffers) === JSON.stringify(offers)
          ) {
            l(`⚠️⚠️⚠️ Warning: No liquidity for collection ${collection}`);
            continue;
          }

          if (offers.length < MAX_OFFERS) {
            l(`⚠️⚠️⚠️ Warning: Add liquidity for collection ${collection}`);
          }

          await h.marketMaker.cwSetOffers(collection, offers, gasPrice);

          collectionAndDateList = collectionAndDateList.map(
            ([currentCollection, updateTime]) => {
              if (currentCollection === collection) {
                return [collection, blockTime + OFFERS_UPDATE_PERIOD];
              }

              return [collection, updateTime];
            }
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
