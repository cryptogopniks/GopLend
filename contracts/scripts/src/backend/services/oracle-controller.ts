import { getSigner } from "../account/signer";
import { getLast, l, li, floor, wait, Request } from "../../common/utils";
import { readFile } from "fs/promises";
import {
  AssetDescription,
  ChainConfig,
  Offer,
  PoolDatabase,
} from "../../common/interfaces";
import { getChainOptionById } from "../../common/config/config-utils";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import { QueryCollectionsResponseItem } from "../../common/codegen/LendingPlatform.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
  specifyTimeout,
} from "./utils";
import {
  CurrencyForToken,
  RawPriceItem,
} from "../../common/codegen/Oracle.types";

const PRICE_RATIO_MULTIPLIER = 0.97;
const PAGINATION_QUERY_AMOUNT = 50;
const QUERY_OFFERS_BATCH_SIZE = 3;

function getLocalBlockTime(): number {
  return floor(Date.now() / 1e3);
}

function getBlockTime(blockTimeOffset: number): number {
  return blockTimeOffset + getLocalBlockTime();
}

function calcAverage(list: number[]): number {
  return list.reduce((acc, cur) => acc + cur, 0) / list.length;
}

function calcCollectionPrice(priceList: number[]): number {
  const offersCount = priceList.length;
  const amounts = priceList.sort((a, b) => b - a); // sort offers by amount descending
  const averageAmount = calcAverage(amounts.slice(0, 3));
  const [maxAmount] = amounts;
  const isLowDeviation = amounts
    .slice(0, 3)
    .every((amount) => 1 - amount / maxAmount <= 0.05);

  if (!offersCount) {
    return 0;
  }

  if (offersCount === 1) {
    return 0.5 * averageAmount;
  }

  if (offersCount === 2) {
    return 0.7 * averageAmount;
  }

  if (offersCount >= 3 && isLowDeviation) {
    return 0.96 * averageAmount;
  }

  return 0.92 * maxAmount;
}

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
      await lending.pQueryCollectionList("main", PAGINATION_QUERY_AMOUNT)
    ).map((x) => x.address);
    const collectionListStable = (
      await lending.pQueryCollectionList("stable", PAGINATION_QUERY_AMOUNT)
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
    batchSize: number
  ): Promise<[string, Offer[]][]> => {
    let collectionAndOffersList: [string, Offer[]][] = [];
    let collectionBatch: string[] = [];

    const addCollection = async (collection: string) => {
      try {
        const offers = await utils.cwQueryMarketplaceOffers(
          marketplaceAddress,
          collection
        );

        collectionAndOffersList.push([collection, offers]);
      } catch (error) {
        l(error);
      }
    };

    // group queries
    for (const collection of collectionList) {
      if (collectionBatch.length !== batchSize) {
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
    blockTimeOffset: number
  ) => {
    const collectionAndOffersList = await queryAllOffers(
      marketplaceAddress,
      collectionList,
      QUERY_OFFERS_BATCH_SIZE
    );

    // remove expired offers and collections w/o offers, calc collection prices
    const blockTime = getBlockTime(blockTimeOffset);
    const collectionAndPriceList = (
      collectionAndOffersList.map(([collection, offers]) => {
        const priceList = offers
          .filter((x) => x.expires_at > blockTime)
          .map((x) => x.price);
        const collectionPrice = calcCollectionPrice(priceList);

        return [collection, collectionPrice];
      }) as [string, number][]
    ).filter(([_collection, collectionPrice]) => collectionPrice);

    // try get stars price from imperator api or pools
    const starsPrice = await queryStarsPrice(req);

    // price_ratio must be decreased offchain due to price_impact
    const priceRatio = floor(starsPrice * PRICE_RATIO_MULTIPLIER, 12);

    // prepare input data for set prices tx
    const priceData: RawPriceItem[] = collectionAndPriceList.map(
      ([collection, collectionPrice]) => {
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

    // li({ priceRatio, priceData });

    // send tx
    await h.oracle.cwSetPrices(priceRatio, priceData, gasPrice);
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

  // loop
  while (true) {
    const date = getLocalBlockTime();
    try {
      await setPrices(
        mainCurrency,
        stableCurrency,
        marketplaceAddress,
        collectionList,
        blockTimeOffset
      );
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
