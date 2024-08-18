import { l } from "../../common/utils";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  CommonConfig,
  LiquidationItem,
  RateConfig,
} from "../../common/codegen/LendingPlatform.types";
import {
  calcLiquidationSet,
  calcLiquidationValue,
  getCollectionAndBidList,
  getCollectionsFromLtvAndCollateralList,
  getTargetBorrowers,
  matchCollateralsAndBids,
  reduceBids,
} from "./math";
import { getBlockTime } from "./utils";

// TODO:
// +check it's safe to use utils.cwGetMarketplaceOffers
// handle both platforms
// +migrate oracle
// +migrate scheduler adapter
// migrate marketplace adapter
// +remove execution cooldown from scheduller adapter
// add query frequency limiter
// can multiple users share same collection offer?
export async function queryTargets(
  chainId: string,
  rpc: string,
  platformType: PlatformType,
  rateConfig: RateConfig,
  commonConfig: CommonConfig,
  ltvMax: number,
  blockTimeOffset: number, // several seconds to consider query delays
  colateralPaginationAmount: number,
  borrowersPaginationAmount: number,
  pricesPaginationAmount: number,
  bidsPaginationAmount: number
): Promise<LiquidationItem[]> {
  const { lending, oracle, utils } = await getCwQueryHelpers(chainId, rpc);

  const collateralList = await lending.pQueryCollateralList(
    platformType,
    colateralPaginationAmount
  );
  const borrowerList = await lending.pQueryBorrowerList(
    platformType,
    borrowersPaginationAmount
  );
  const priceList = await oracle.pQueryPrices(pricesPaginationAmount);
  const blockTime = getBlockTime(blockTimeOffset);

  // get borrowers with ltv > ltvMax, sort by ltv descending
  const ltvAndCollateralList = getTargetBorrowers(
    borrowerList,
    collateralList,
    priceList,
    rateConfig,
    commonConfig,
    ltvMax,
    blockTime
  );

  if (!ltvAndCollateralList.length) {
    return [];
  }

  const collectionAddressList =
    getCollectionsFromLtvAndCollateralList(ltvAndCollateralList);

  const liquidationBids =
    await lending.pQueryLiquidationBidsByCollectionAddressList(
      platformType,
      bidsPaginationAmount
    );
  const marketplaceOffers = await utils.cwGetMarketplaceOffers(
    platformType,
    collectionAddressList,
    commonConfig.is_marketplace_interface_enabled
  );
  let totalBids = [...liquidationBids, ...marketplaceOffers];

  return ltvAndCollateralList.reduce(
    (
      acc,
      {
        borrowerAddress,
        ltv,
        loanAmount,
        loanCreationDate,
        accumulatedLoan,
        collateralValue,
        collectionAndCollateralList,
      }
    ) => {
      const collectionAndBidList = getCollectionAndBidList(
        totalBids,
        collectionAndCollateralList
      );

      const biddedCollateralList = matchCollateralsAndBids(
        commonConfig.main_currency,
        collectionAndCollateralList,
        collectionAndBidList,
        priceList
      );

      // calculate how much tokens and what collection must be liquidated to get proper ltv
      // for worst case (max discount)
      const maxLiquidationValue = calcLiquidationValue(
        collateralValue,
        Number(rateConfig.discount_max_rate),
        ltv,
        ltvMax
      );

      const liquidationSet = calcLiquidationSet(
        biddedCollateralList,
        maxLiquidationValue,
        ltvMax,
        Number(rateConfig.borrow_apr),
        accumulatedLoan,
        loanAmount,
        collateralValue,
        blockTime,
        loanCreationDate
      );

      if (liquidationSet.length) {
        // remove used bids from totalBids
        totalBids = reduceBids(totalBids, liquidationSet);

        acc.push({
          borrower: borrowerAddress,
          liquidation_set: liquidationSet,
        });
      } else {
        l(
          `⚠️⚠️⚠️ Warning: No (proper) bids/offers to liquidate ${borrowerAddress}`
        );
      }

      return acc;
    },
    [] as LiquidationItem[]
  );
}
