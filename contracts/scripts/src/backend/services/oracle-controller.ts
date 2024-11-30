import { getSigner } from "../account/signer";
import { l, li, floor, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { getChainOptionById } from "../../common/config/config-utils";
import { getFloorPrices } from "./sg-api";
import { ChainConfig, ChainType } from "../../common/interfaces";
import { COLLECTION, NetworkItem } from "./constants";
import {
  calcCollectionPrice,
  CollectionPriceResponse,
  PriceSample,
} from "./math";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
  getLocalBlockTime,
  getBlockTime,
  getStargazeCollections,
} from "./utils";

// 1) +query FP using sg-api
// 2) get valid range
// 3) query mm offers in valid range
// 4) query token prices
// 5) calculate prices for oracle using collection offers
// 6) query oracle prices
// 7) don't update if offers price has changed less than 1 % per iteration but update at least once per 1 min

interface CollectionInfo {
  collection: string;
  price: number;
  collectionPriceResponse: CollectionPriceResponse;
}

const BLOCK_TIME_MARGIN = 10; // delay in seconds to consider paginated queries time difference

const PAGINATION = {
  COLLECTIONS: 50,
};

const TWAP_WEIGHT: number = 0.7;
const BLOCK_TIME: number = 6;
const SAMPLING_WINDOW: number = 10 * BLOCK_TIME;
const FP_TWAP_WINDOW_MULTIPLIER: number = 2;
const FLOOR_PRICE_BOUNDARY = {
  UPPER: 0.93,
  LOWER: 0.8,
};

const COLLECTION_PRICE_RESPONSE_DEFAULT: CollectionPriceResponse = {
  twapSampleList: [],
  floorPriceEma: 0,
  floorPrice: 0,
  twapOfferList: [],
  offerEma: 0,
  collectionPrice: 0,
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

  const { lending, oracle, marketMaker } = await getCwQueryHelpers(
    chainId,
    RPC
  );
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);

  const getCollectionList = async () => {
    const collectionList = (
      await lending.pQueryCollectionList(PAGINATION.COLLECTIONS)
    ).map((x) => x.address);

    return collectionList;
  };

  const getInitialParameters = async (chainType: ChainType) => {
    // get execution cooldown
    const executionCooldown = (await oracle.cwQueryConfig()).execution_cooldown;

    // get all listed collections
    const collectionList = await getCollectionList();

    // get blockTimeOffset to fix difference between local and network time
    const blockTime = await oracle.cwQueryBlockTime();
    const localBlockTime = getLocalBlockTime();
    const blockTimeOffset = blockTime - localBlockTime;

    const mainnetCollectionEntries = Object.entries(COLLECTION?.MAINNET || {});
    const collectionEntries =
      chainType === "main"
        ? mainnetCollectionEntries
        : Object.entries(COLLECTION?.TESTNET || {});

    return {
      executionCooldown,
      collectionList,
      blockTimeOffset,
      mainnetCollectionEntries,
      collectionEntries,
    };
  };

  const setPrices = async (
    mainnetCollectionEntries: [string, NetworkItem][],
    collectionEntries: [string, NetworkItem][],
    collectionList: string[],
    blockTimeOffset: number,
    collectionPriceResponseList: [string, CollectionPriceResponse][]
  ): Promise<[string, CollectionPriceResponse][]> => {
    let blockTime = getBlockTime(blockTimeOffset) + BLOCK_TIME_MARGIN;

    const stargazeCollectionList = getStargazeCollections(
      collectionList,
      mainnetCollectionEntries,
      collectionEntries
    );

    const collectionAndFloorPriceList = await getFloorPrices(
      stargazeCollectionList
    );
    const collectionAndOffersList = await marketMaker.pQueryOfferPricesList(
      PAGINATION.COLLECTIONS
    );

    // calc collection prices
    const collectionInfoList: CollectionInfo[] = collectionList.map(
      (collection, i) => {
        const stargazeCollection = stargazeCollectionList[i];

        const offerPriceList =
          collectionAndOffersList
            .find((x) => x.collection_address === collection)
            ?.price_list.map(Number) || [];

        // use highest offer if floor price isn't found
        const floorPrice =
          collectionAndFloorPriceList.find(
            (x) => x.collection === stargazeCollection
          )?.price ||
          offerPriceList[0] ||
          0;

        const [_, collectionPriceResponse] = collectionPriceResponseList.find(
          ([collectionAddress]) => collectionAddress === collection
        ) || [collection, COLLECTION_PRICE_RESPONSE_DEFAULT];

        const {
          twapSampleList: floorPriceSampleList,
          floorPriceEma: prevFloorPriceEma,
          twapOfferList: meanOfferSampleList,
          offerEma: prevMeanOfferEma,
        } = collectionPriceResponse;

        const newFloorPriceSample: PriceSample = {
          value: floorPrice,
          timestamp: blockTime,
        };

        const {
          twapSampleList,
          floorPriceEma,
          twapOfferList,
          offerEma,
          collectionPrice,
        } = calcCollectionPrice(
          FLOOR_PRICE_BOUNDARY.UPPER,
          FLOOR_PRICE_BOUNDARY.LOWER,
          TWAP_WEIGHT,
          SAMPLING_WINDOW,
          FP_TWAP_WINDOW_MULTIPLIER,
          floorPriceSampleList,
          newFloorPriceSample,
          prevFloorPriceEma,
          offerPriceList,
          meanOfferSampleList,
          prevMeanOfferEma
        );

        return {
          collection,
          price: collectionPrice,
          collectionPriceResponse: {
            twapSampleList,
            floorPriceEma,
            twapOfferList,
            offerEma,
            collectionPrice,
          },
        };
      }
    ) as CollectionInfo[];

    // prepare input data for set prices tx
    const priceData: [string, number][] = collectionInfoList.map(
      ({ collection, price }) => [collection, floor(price, 2)]
    );

    // send tx
    await h.oracle.cwUpdatePrices(priceData, gasPrice);

    return collectionInfoList.map((x) => [
      x.collection,
      x.collectionPriceResponse,
    ]);
  };

  // TODO: restart script on collection list change
  // initialization part - run it single time on script start
  const {
    executionCooldown,
    collectionList,
    blockTimeOffset,
    mainnetCollectionEntries,
    collectionEntries,
  } = await getInitialParameters(TYPE);

  // [collection, CollectionPriceResponse][]
  let collectionPriceResponseList: [string, CollectionPriceResponse][] = [];

  // loop
  while (true) {
    const date = getLocalBlockTime();

    try {
      // TODO: query collectionPriceResponseList from DB
      // collectionPriceResponseList = await queryCollectionPriceResponseList()
    } catch (error) {
      l(error, "\n");
    }

    try {
      collectionPriceResponseList = await setPrices(
        mainnetCollectionEntries,
        collectionEntries,
        collectionList,
        blockTimeOffset,
        collectionPriceResponseList
      );

      // TODO: write collectionPriceResponseList in DB
      // await setCollectionPriceResponseList(collectionPriceResponseList)
    } catch (error) {
      l(error, "\n");
      await wait(executionCooldown * 1e3);
    }
    l({ cycleLength: getLocalBlockTime() - date });

    if (!isInfiniteLoop) break;
  }
}

// TODO: set true for production
main(true);
