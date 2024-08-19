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
import { getBlockTime, wrapListQuerier } from "./utils";

const platformList: PlatformType[] = ["main", "stable"];

// TODO:
// +check it's safe to use utils.cwGetMarketplaceOffers
// handle both platforms
// +migrate oracle
// +migrate scheduler adapter
// migrate marketplace adapter
// +remove execution cooldown from scheduller adapter
// +-add query frequency limiter
// can multiple users share same collection offer?
export async function queryTargets(
  chainId: string,
  rpc: string,
  rateConfigMain: RateConfig,
  rateConfigStable: RateConfig,
  commonConfigMain: CommonConfig,
  commonConfigStable: CommonConfig,
  ltvMaxMain: number,
  ltvMaxStable: number,
  blockTimeOffset: number, // several seconds to consider query delays
  colateralPaginationAmount: number,
  borrowersPaginationAmount: number,
  pricesPaginationAmount: number,
  bidsPaginationAmount: number
): Promise<LiquidationItem[]> {
  const { lending, oracle, utils } = await getCwQueryHelpers(chainId, rpc);

  const [collateralListMain, collateralListStable] = await Promise.all(
    platformList.map((platformType) =>
      wrapListQuerier(
        lending.pQueryCollateralList,
        platformType,
        colateralPaginationAmount
      )
    )
  );
  const [borrowerListMain, borrowerListStable] = await Promise.all(
    platformList.map((platformType) =>
      wrapListQuerier(
        lending.pQueryBorrowerList,
        platformType,
        borrowersPaginationAmount
      )
    )
  );
  const priceList = await oracle.pQueryPrices(pricesPaginationAmount);
  const blockTime = getBlockTime(blockTimeOffset);

  // get borrowers with ltv > ltvMax, sort by ltv descending
  const [ltvAndCollateralListMain, ltvAndCollateralListStable] =
    platformList.map((platformType) =>
      getTargetBorrowers(
        platformType,
        platformType === "main" ? borrowerListMain : borrowerListStable,
        platformType === "main" ? collateralListMain : collateralListStable,
        priceList,
        platformType === "main" ? rateConfigMain : rateConfigStable,
        platformType === "main" ? commonConfigMain : commonConfigStable,
        platformType === "main" ? ltvMaxMain : ltvMaxStable,
        blockTime
      )
    );

  // sort by ltv descending
  const ltvAndCollateralList = [
    ...ltvAndCollateralListMain,
    ...ltvAndCollateralListStable,
  ].sort(({ ltv: ltvA }, { ltv: ltvB }) => ltvB - ltvA);

  if (!ltvAndCollateralList.length) {
    return [];
  }

  const [liquidationBidsMain, liquidationBidsStable] = await Promise.all(
    platformList.map((platformType) =>
      wrapListQuerier(
        lending.pQueryLiquidationBidsByCollectionAddressList,
        platformType,
        bidsPaginationAmount
      )
    )
  );

  const [marketplaceOffersMain, marketplaceOffersStable] = await Promise.all(
    platformList.map((platformType) =>
      wrapListQuerier(
        utils.cwGetMarketplaceOffers,
        platformType,
        getCollectionsFromLtvAndCollateralList(
          platformType === "main"
            ? ltvAndCollateralListMain
            : ltvAndCollateralListStable
        ),
        (platformType === "main" ? commonConfigMain : commonConfigStable)
          .is_marketplace_interface_enabled
      )
    )
  );

  let totalBidsMain = [...liquidationBidsMain, ...marketplaceOffersMain];
  let totalBidsStable = [...liquidationBidsStable, ...marketplaceOffersStable];

  // const liquidationTargets = ltvAndCollateralList.reduce(
  //   (
  //     acc,
  //     {
  //       platformType,
  //       borrowerAddress,
  //       ltv,
  //       loanAmount,
  //       loanCreationDate,
  //       accumulatedLoan,
  //       collateralValue,
  //       collectionAndCollateralList,
  //     }
  //   ) => {
  //     const collectionAndBidList = getCollectionAndBidList(
  //       totalBids,
  //       collectionAndCollateralList
  //     );

  //     const biddedCollateralList = matchCollateralsAndBids(
  //       commonConfig.main_currency,
  //       collectionAndCollateralList,
  //       collectionAndBidList,
  //       priceList
  //     );

  //     // calculate how much tokens and what collection must be liquidated to get proper ltv
  //     // for worst case (max discount)
  //     const maxLiquidationValue = calcLiquidationValue(
  //       collateralValue,
  //       Number(rateConfig.discount_max_rate),
  //       ltv,
  //       ltvMax
  //     );

  //     const liquidationSet = calcLiquidationSet(
  //       biddedCollateralList,
  //       maxLiquidationValue,
  //       ltvMax,
  //       Number(rateConfig.borrow_apr),
  //       accumulatedLoan,
  //       loanAmount,
  //       collateralValue,
  //       blockTime,
  //       loanCreationDate
  //     );

  //     if (liquidationSet.length) {
  //       // remove used bids from totalBids
  //       totalBids = reduceBids(totalBids, liquidationSet);

  //       acc.push({
  //         borrower: borrowerAddress,
  //         liquidation_set: liquidationSet,
  //       });
  //     } else {
  //       l(
  //         `⚠️⚠️⚠️ Warning: No (proper) bids/offers to liquidate ${borrowerAddress}`
  //       );
  //     }

  //     return acc;
  //   },
  //   [] as LiquidationItem[]
  // );

  return [];
}
