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
  getTargetBorrowers,
  matchCollateralsAndBids,
  reduceBids,
} from "./math";

// TODO: single query
//  const [rateConfig, commonConfig] = await Promise.all([
//   lending.cwQueryRateConfig(platformType, true),
//   lending.cwQueryCommonConfig(platformType, true),
// ]);

// const ltvMax = calcLtvMax(
//   Number(rateConfig.bid_min_rate),
//   Number(rateConfig.discount_max_rate),
//   Number(rateConfig.discount_min_rate)
// );

const PAGINATION_AMOUNT = 100;

// TODO: handle both platforms
// TODO: migrate marketplace adapter
// TODO: remove execution cooldown from scheduller controller
// TODO: add query frequency limiter
// TODO: can multiple users share same collection offer?
export async function queryTargets(
  chainId: string,
  rpc: string,
  platformType: PlatformType,
  rateConfig: RateConfig,
  commonConfig: CommonConfig,
  ltvMax: number,
  blockTime: number // must be increased by several seconds to consider query delays
): Promise<LiquidationItem[]> {
  const { lending, oracle, utils } = await getCwQueryHelpers(chainId, rpc);

  const collateralList = await lending.pQueryCollateralList(
    platformType,
    PAGINATION_AMOUNT
  );
  const borrowerList = await lending.pQueryBorrowerList(
    platformType,
    PAGINATION_AMOUNT
  );
  const priceList = await oracle.pQueryPrices(PAGINATION_AMOUNT);

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

  const collectionAddressList = ltvAndCollateralList.reduce(
    (acc, { collectionAndCollateralList }) => {
      collectionAndCollateralList.forEach(([collectionAddress]) => {
        if (!acc.includes(collectionAddress)) {
          acc.push(collectionAddress);
        }
      });

      return acc;
    },
    [] as string[]
  );

  const liquidationBids =
    await lending.pQueryLiquidationBidsByCollectionAddressList(
      platformType,
      PAGINATION_AMOUNT
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
