import { getSigner } from "../account/signer";
import { l, li, floor, wait, Request } from "../../common/utils";
import { readFile } from "fs/promises";
import { getChainOptionById } from "../../common/config/config-utils";
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
  CurrencyForToken,
  RawPriceItem,
} from "../../common/codegen/Oracle.types";
import {
  AssetDescription,
  ChainConfig,
  Offer,
  PoolDatabase,
} from "../../common/interfaces";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
  specifyTimeout,
  getLocalBlockTime,
  getBlockTime,
} from "./utils";

interface CollectionInfo {
  collection: string;
  price: number;
  collectionPriceResponse: CollectionPriceResponse;
}

const BLOCK_TIME_MARGIN = 10; // delay in seconds to consider paginated queries time difference
const PRICE_RATIO_MULTIPLIER = 0.97; // to consider swap price impact

const PAGINATION = {
  COLLECTIONS: 50,
  OFFERS: {
    COLLECTION_BATCH: 5,
    COLLECTION_LIMIT: 50,
    COLLECTION_MAX_COUNT: 50,
  },
};

const FLOOR_PRICE_BOUNDARY: number = 0.75;
const TWAP_WEIGHT: number = 0.7;
const BLOCK_TIME: number = 6;
const SAMPLING_WINDOW: number = 10 * BLOCK_TIME;
const OFFER_TO_FP_RATIO: number = 0.9;

const COLLECTION_PRICE_RESPONSE_DEFAULT: CollectionPriceResponse = {
  twapSampleList: [],
  floorPriceEma: 0,
  twapOfferList: [],
  offerEma: 0,
  collectionPrice: 0,
};

async function queryStarsPriceFromImperator(req: Request) {
  const starsSymbol = "STARS";
  const priceUrl =
    "https://api-osmosis.imperator.co/pools/v2/all?low_liquidity=false";
  const poolDatabase: PoolDatabase = await req.get(priceUrl);

  // get all stars pools
  const [{ price: starsPrice }] = Object.entries(poolDatabase)
    .filter(([_, assetList]) => assetList.some((x) => x.symbol === starsSymbol))
    .map(
      ([_, assetList]) =>
        assetList.find((x) => x.symbol === starsSymbol) as AssetDescription
    )
    .sort((a, b) => b.amount - a.amount);

  return starsPrice;
}

async function querySpotPrice(
  req: Request,
  lcd: string,
  poolId: number,
  askDenom: string,
  offerDenom: string
) {
  interface SpotPrice {
    spot_price: string;
  }

  const spotPrice: SpotPrice = await req.get(
    `${lcd}/osmosis/gamm/v1beta1/pools/${poolId}/prices`,
    {
      params: {
        base_asset_denom: askDenom,
        quote_asset_denom: offerDenom,
      },
    }
  );

  return Number(spotPrice.spot_price);
}

async function queryStarsPriceFromPools(req: Request) {
  const lcd = "https://rest-osmosis.ecostake.com";
  const poolStarsOsmo = 604;
  const poolUsdcOsmo = 1369;
  const osmoDenom = "uosmo";
  const starsDenom =
    "ibc/987C17B11ABC2B20019178ACE62929FE9840202CE79498E29FE8E5CB02B7C0A4";
  const usdcDenom =
    "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4";

  const [osmoForUsdc, starsForOsmo] = await Promise.all([
    querySpotPrice(req, lcd, poolUsdcOsmo, usdcDenom, osmoDenom),
    querySpotPrice(req, lcd, poolStarsOsmo, osmoDenom, starsDenom),
  ]);

  return osmoForUsdc * starsForOsmo;
}

async function queryStarsPrice(req: Request): Promise<number> {
  const fn1 = async (): Promise<number | undefined> => {
    try {
      return await specifyTimeout(queryStarsPriceFromImperator(req));
    } catch (error) {
      l(error);
    }
  };

  const fn2 = async (): Promise<number | undefined> => {
    try {
      return await specifyTimeout(queryStarsPriceFromPools(req));
    } catch (error) {
      l(error);
    }
  };

  const [starsPriceImperator, starsPricePools] = await Promise.all([
    fn1(),
    fn2(),
  ]);

  const starsPrice = starsPriceImperator || starsPricePools;
  if (!starsPrice) throw new Error("Stars price isn't found!");

  return starsPrice;
}

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

  const { lending, adapter, oracle, utils } = await getCwQueryHelpers(
    chainId,
    RPC
  );
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);
  const req = new Request();

  const getCollectionList = async () => {
    const collectionListMain = (
      await lending.pQueryCollectionList("main", PAGINATION.COLLECTIONS)
    ).map((x) => x.address);
    const collectionListStable = (
      await lending.pQueryCollectionList("stable", PAGINATION.COLLECTIONS)
    ).map((x) => x.address);

    return Array.from(
      new Set([...collectionListMain, ...collectionListStable])
    );
  };

  const getInitialParameters = async () => {
    // get execution cooldown
    const executionCooldown = (await oracle.cwQueryConfig()).execution_cooldown;

    // get currencies
    const mainCurrency = (await lending.cwQueryCommonConfig("main"))
      .main_currency;
    const stableCurrency = (await lending.cwQueryCommonConfig("stable"))
      .main_currency;

    // get stargaze marketplace address
    const { marketplace: marketplaceAddress } =
      await adapter.marketplace.stargaze.cwQueryConfig();
    if (!marketplaceAddress) throw new Error("Marketplace isn't found!");

    // get all listed collections
    const collectionList = await getCollectionList();

    // get blockTimeOffset to fix difference between local and network time
    const blockTime = await oracle.cwQueryBlockTime();
    const localBlockTime = getLocalBlockTime();
    const blockTimeOffset = blockTime - localBlockTime;

    return {
      executionCooldown,
      mainCurrency,
      stableCurrency,
      marketplaceAddress,
      collectionList,
      blockTimeOffset,
    };
  };

  const queryAllOffers = async (
    marketplaceAddress: string,
    collectionList: string[],
    collectionBatchSize: number,
    collectionLimit: number,
    collectionMaxCount: number,
    blockTime: number
  ): Promise<[string, Offer[]][]> => {
    let collectionAndOffersList: [string, Offer[]][] = [];
    let collectionBatch: string[] = [];

    const addCollection = async (collection: string) => {
      try {
        const offers: Offer[] = (
          await utils.pQueryMarketplaceCollectionOffers(
            marketplaceAddress,
            collection,
            blockTime,
            collectionLimit,
            collectionMaxCount
          )
        ).map((x) => ({
          price: Number(x.price),
          expires_at: Math.floor(Number(x.expires_at) / 1e9),
        }));

        collectionAndOffersList.push([collection, offers]);
      } catch (error) {
        l(error);
      }
    };

    // group queries
    for (const collection of collectionList) {
      if (collectionBatch.length !== collectionBatchSize) {
        collectionBatch.push(collection);
      } else {
        await Promise.all(
          collectionBatch.map((collection) => addCollection(collection))
        );

        collectionBatch = [collection];
      }
    }

    await Promise.all(
      collectionBatch.map((collection) => addCollection(collection))
    );

    return collectionAndOffersList;
  };

  const setPrices = async (
    mainCurrency: CurrencyForToken,
    stableCurrency: CurrencyForToken,
    marketplaceAddress: string,
    collectionList: string[],
    blockTimeOffset: number,
    collectionPriceResponseList: [string, CollectionPriceResponse][]
  ): Promise<[string, CollectionPriceResponse][]> => {
    let blockTime = getBlockTime(blockTimeOffset) + BLOCK_TIME_MARGIN;
    const collectionAndOffersList = await queryAllOffers(
      marketplaceAddress,
      collectionList,
      PAGINATION.OFFERS.COLLECTION_BATCH,
      PAGINATION.OFFERS.COLLECTION_LIMIT,
      PAGINATION.OFFERS.COLLECTION_MAX_COUNT,
      blockTime
    );

    const floorPriceList: [string, number][] = (
      await adapter.marketplace.stargaze.cwQueryFloorPrices(collectionList)
    ).map(([collectionAddress, floorPriceString]) => {
      const floorPrice = Number(floorPriceString);

      if (floorPrice) {
        return [collectionAddress, floorPrice];
      }

      const [_, offers] =
        collectionAndOffersList.find(
          ([collection]) => collection === collectionAddress
        ) || [];

      const highestOffer = (offers || [])[0].price || 0;

      return [collectionAddress, highestOffer / OFFER_TO_FP_RATIO];
    });

    // calc collection prices
    const collectionInfoList: CollectionInfo[] = collectionAndOffersList.map(
      ([collection, offers]) => {
        const offerPriceList = offers.map((x) => x.price);

        const [_, collectionPriceResponse] = collectionPriceResponseList.find(
          ([collectionAddress]) => collectionAddress === collection
        ) || [collection, COLLECTION_PRICE_RESPONSE_DEFAULT];

        const {
          twapSampleList: floorPriceSampleList,
          floorPriceEma: prevFloorPriceEma,
          twapOfferList: medianOfferSampleList,
          offerEma: prevMedianOfferEma,
        } = collectionPriceResponse;

        const [_collectionAddress, floorPrice] = floorPriceList.find(
          ([collectionAddress]) => collectionAddress === collection
        ) || [collection, 0];

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
          FLOOR_PRICE_BOUNDARY,
          TWAP_WEIGHT,
          SAMPLING_WINDOW,
          floorPriceSampleList,
          newFloorPriceSample,
          prevFloorPriceEma,
          offerPriceList,
          medianOfferSampleList,
          prevMedianOfferEma
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

    // try get stars price from imperator api or pools
    const starsPrice = await queryStarsPrice(req);

    // price_ratio must be decreased offchain due to price_impact
    const priceRatio = floor(starsPrice * PRICE_RATIO_MULTIPLIER, 12);

    // prepare input data for set prices tx
    const priceData: RawPriceItem[] = collectionInfoList.map(
      ({ collection, price: collectionPrice }) => {
        return {
          collection_address: collection,
          price: {
            amount: floor(collectionPrice).toString(),
            currency: mainCurrency,
          },
          usd_price: {
            amount: floor(priceRatio * collectionPrice).toString(),
            currency: stableCurrency,
          },
        };
      }
    );

    // send tx
    await h.oracle.cwSetPrices(priceRatio, priceData, gasPrice);

    return collectionInfoList.map((x) => [
      x.collection,
      x.collectionPriceResponse,
    ]);
  };

  // TODO: restart script on collection list change
  // initialization part - run it single time on script start
  const {
    executionCooldown,
    mainCurrency,
    stableCurrency,
    marketplaceAddress,
    collectionList,
    blockTimeOffset,
  } = await getInitialParameters();

  // [collection, CollectionPriceResponse][]
  let collectionPriceResponseList: [string, CollectionPriceResponse][] = [];

  try {
    // TODO: query collectionPriceResponseList from DB
    // collectionPriceResponseList = await queryCollectionPriceResponseList()
  } catch (error) {
    l(error);
  }

  // loop
  while (true) {
    const date = getLocalBlockTime();
    try {
      collectionPriceResponseList = await setPrices(
        mainCurrency,
        stableCurrency,
        marketplaceAddress,
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
