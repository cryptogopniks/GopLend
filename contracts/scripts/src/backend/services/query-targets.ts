import { l, li } from "../../common/utils";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import { getBlockTime } from "./utils";
import {
  LiquidationItem,
  RateConfig,
} from "../../common/codegen/LendingPlatform.types";
import {
  aggregateTokensByCollection,
  calcLiquidationSet,
  calcLiquidationValue,
  distributeOffers,
  getCollectionAndBidList,
  getCollectionsFromLtvAndCollateralList,
  getTargetBorrowers,
  matchCollateralsAndBids,
  mergeBidsAndOffers,
  reduceBids,
} from "./math";

export async function queryTargets(
  chainId: string,
  rpc: string,
  rateConfig: RateConfig,
  ltvMax: number,
  ltvMaxFraction: number,
  blockTimeOffset: number, // several seconds to consider query delays
  collateralPaginationAmount: number,
  borrowersPaginationAmount: number,
  pricesPaginationAmount: number,
  bidsPaginationAmount: number,
  offersBatchPaginationAmount: number // 5
): Promise<LiquidationItem[]> {
  const { lending, oracle, utils } = await getCwQueryHelpers(chainId, rpc);

  const collateralList = await lending.pQueryCollateralList(
    collateralPaginationAmount
  );
  const borrowerList = await lending.pQueryBorrowerList(
    borrowersPaginationAmount
  );
  const priceList = await oracle.pQueryPrices(pricesPaginationAmount);
  const blockTime = getBlockTime(blockTimeOffset);

  // handle outdated prices
  if (priceList.is_outdated) {
    l(`⚠️⚠️⚠️ Warning: prices are outdated`);
    return [];
  }

  // get borrowers with ltv > ltvMax, sort by ltv descending
  const ltvAndCollateralList = getTargetBorrowers(
    borrowerList,
    collateralList,
    priceList.data,
    rateConfig,
    ltvMax,
    blockTime
  ).sort(({ ltv: ltvA }, { ltv: ltvB }) => ltvB - ltvA);

  if (!ltvAndCollateralList.length) {
    return [];
  }

  const collectionList =
    getCollectionsFromLtvAndCollateralList(ltvAndCollateralList);

  const liquidationBids =
    await lending.pQueryLiquidationBidsByCollectionAddressList(
      bidsPaginationAmount
    );

  // const collectionAndTokensList = aggregateTokensByCollection(
  //   ltvAndCollateralList,
  //   collectionList
  // );

  const collectionOfferList = await utils.cwGetMarketMakerData(
    collectionList,
    offersBatchPaginationAmount
  );

  const offers = distributeOffers(
    collectionList,
    collateralList,
    collectionOfferList,
    blockTime
  );

  let totalBids = mergeBidsAndOffers(liquidationBids, offers);

  const liquidationTargets: LiquidationItem[] = ltvAndCollateralList.reduce(
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
      const { discount_max_rate, borrow_apr } = rateConfig;
      const collectionAndBidList = getCollectionAndBidList(
        totalBids,
        collectionAndCollateralList
      );

      const biddedCollateralList = matchCollateralsAndBids(
        collectionAndCollateralList,
        collectionAndBidList,
        priceList.data
      );

      // calculate how much tokens and what collection must be liquidated to get proper ltv
      // for worst case (max discount)
      const maxLiquidationValue = calcLiquidationValue(
        collateralValue,
        Number(discount_max_rate),
        ltv,
        ltvMax,
        ltvMaxFraction
      );

      const liquidationSet = calcLiquidationSet(
        biddedCollateralList,
        maxLiquidationValue,
        ltvMax,
        Number(borrow_apr),
        accumulatedLoan,
        loanAmount,
        collateralValue,
        blockTime,
        loanCreationDate
      );

      if (liquidationSet.length) {
        // remove used bids
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

  return liquidationTargets;
}
