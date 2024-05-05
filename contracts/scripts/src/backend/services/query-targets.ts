import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ENCODING, PATH_TO_CONFIG_JSON, parseStoreArgs } from "./utils";
import { getChainOptionById } from "../../common/config/config-utils";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import {
  Addr,
  ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
  Borrower,
  Collateral,
  LiquidationBid,
  LiquidationItem,
} from "../../common/codegen/LendingPlatform.types";
import { ArrayOfPriceItem, PriceItem } from "../../common/codegen/Oracle.types";
import {
  calcConditionalLtv,
  calcLiquidationSet,
  calcLiquidationValue,
  calcLtvMax,
  matchCollateralsAndBids,
} from "./math";

export async function queryTargets(
  borrowers: string[]
): Promise<LiquidationItem[]> {
  // configs
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    OPTION: {
      RPC_LIST: [RPC],
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { lending, oracle, adapter } = await getCwQueryHelpers(chainId, RPC);

  // helpers
  const getBorrowers = async (
    borrowers: string[]
  ): Promise<[Addr, Borrower][]> => {
    return (await lending.cwQueryBorrowerList())
      .filter(({ address }) => borrowers.includes(address))
      .map(({ address, borrower }) => [address, borrower]);
  };

  const getCollectionPrices = async (
    borrowers: [Addr, Borrower][],
    collectionAndCollateralList: [Addr, Collateral[]][]
  ): Promise<ArrayOfPriceItem> => {
    // collect collection addresses over current page borrowers
    const borrowerAddresses: Addr[] = borrowers.map(([address, _]) => address);

    let collectionAddressesByBorrowers: string[] = collectionAndCollateralList
      .filter(([_, collaterals]) =>
        collaterals.some(({ owner }) => borrowerAddresses.includes(owner))
      )
      .map(([collectionAddress]) => collectionAddress);

    // dedup
    collectionAddressesByBorrowers = [
      ...new Set(collectionAddressesByBorrowers),
    ];

    return await oracle.cwQueryPrices(
      0,
      undefined,
      collectionAddressesByBorrowers
    );
  };

  const getCollateralsByBorrower = (
    collectionAndCollateralList: [Addr, Collateral[]][],
    borrowerAddress: Addr
  ): [Addr, Collateral][] => {
    // get collaterals owned by borrower
    return collectionAndCollateralList
      .map(([collection, collaterals]) => {
        // single collateral per collection
        const collateral = collaterals.find(
          (collateral) => collateral.owner === borrowerAddress
        );

        return [collection, collateral];
      })
      .filter(([_, collateral]) => collateral) as [Addr, Collateral][];
  };

  const getCollateralValue = (
    collectionAndCollateralListByBorrower: [Addr, Collateral][],
    prices: PriceItem[],
    blockTime: number,
    priceUpdatePeriod: number
  ) => {
    let isPriceOutdated = false;

    const collateralValue = collectionAndCollateralListByBorrower.reduce(
      (acc, [collectionAddress, collateral]) => {
        const priceItem = prices.find((x) => x.address === collectionAddress);
        if (!priceItem) return acc;

        const { price, price_update_date } = priceItem;

        if (blockTime - price_update_date > priceUpdatePeriod) {
          isPriceOutdated = true;
        }

        return acc + Number(price.amount) * collateral.token_id_list.length;
      },
      0
    );

    return !isPriceOutdated ? collateralValue : undefined;
  };

  // find bids containing at least 1 borrower's collateral token
  const getCollectionAndBidList = async (
    collectionAndCollateralListByBorrower: [Addr, Collateral][]
  ): Promise<[Addr, LiquidationBid][]> => {
    const liquidationBids =
      await lending.cwQueryLiquidationBidsByCollectionAddressList();

    let liquidationBidsFromMarketplace: ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem =
      [];
    try {
      liquidationBidsFromMarketplace =
        await adapter.marketplace.kujira.cwQueryLiquidationBidsByCollectionAddressList();
    } catch (error) {
      l(error);
    }

    let collectionAndBidList: [Addr, LiquidationBid][] = [];

    for (const [
      collection_address,
      collateral,
    ] of collectionAndCollateralListByBorrower) {
      const liquidationBidsByCollectionAddress = [
        ...liquidationBids,
        ...liquidationBidsFromMarketplace,
      ].find((x) => x.collection_address === collection_address);
      if (!liquidationBidsByCollectionAddress) continue;

      for (const bid of liquidationBidsByCollectionAddress.liquidation_bids) {
        if (
          bid.token_id_list.some((id) => collateral.token_id_list.includes(id))
        ) {
          collectionAndBidList.push([collection_address, bid]);
        }
      }
    }

    return collectionAndBidList;
  };

  // core logic
  const rateConfig = await lending.cwQueryRateConfig();
  const commonConfig = await lending.cwQueryCommonConfig();
  const borrowerList = await getBorrowers(borrowers);

  // get all collaterals
  const collectionAndCollateralList: [Addr, Collateral[]][] = (
    await lending.cwQueryCollateralList()
  ).map(({ address, collateral }) => [address, collateral]);

  // get collection prices
  const prices = await getCollectionPrices(
    borrowerList,
    collectionAndCollateralList
  );
  const blockTime = await oracle.cwQueryBlockTime();

  // accumulators
  let targets: LiquidationItem[] = [];

  // iterate over borrowers
  for (const [borrowerAddress, { accumulated_loan, loan }] of borrowerList) {
    // get collaterals owned by borrower
    const collectionAndCollateralListByBorrower = getCollateralsByBorrower(
      collectionAndCollateralList,
      borrowerAddress
    );

    const collateralValueOption = getCollateralValue(
      collectionAndCollateralListByBorrower,
      prices,
      blockTime,
      commonConfig.price_update_period
    );

    // go to next borrower if any price is outdated
    if (!collateralValueOption) {
      continue;
    }

    const collateralValue = collateralValueOption;

    // check ltv
    const ltvMax = calcLtvMax(
      Number(rateConfig.bid_min_rate),
      Number(rateConfig.discount_max_rate),
      Number(rateConfig.discount_min_rate)
    );

    const ltv = calcConditionalLtv(
      Number(rateConfig.borrow_apr),
      0,
      Number(accumulated_loan),
      Number(loan.amount),
      0,
      collateralValue,
      blockTime,
      loan.creation_date
    );

    // if ltv is low check next borrower, otherwise liquidate required amount
    if (ltv <= ltvMax) {
      continue;
    }

    // find bids containing at least 1 borrower's collateral token
    const collectionAndBidList = await getCollectionAndBidList(
      collectionAndCollateralListByBorrower
    );

    const biddedCollateralList = matchCollateralsAndBids(
      collectionAndCollateralListByBorrower,
      collectionAndBidList,
      prices
    );

    // calculate how much tokens and what collection must be liquidated to get proper ltv
    // for worst case (max discount)
    const maxLiquidationValue = calcLiquidationValue(
      collateralValue,
      Number(rateConfig.discount_max_rate),
      ltv,
      ltvMax
    );

    const liquidation_set = calcLiquidationSet(
      biddedCollateralList,
      maxLiquidationValue,
      ltvMax,
      Number(rateConfig.borrow_apr),
      Number(accumulated_loan),
      Number(loan.amount),
      collateralValue,
      blockTime,
      loan.creation_date
    );

    targets.push({
      borrower: borrowerAddress,
      liquidation_set,
    });
  }

  return targets;
}
