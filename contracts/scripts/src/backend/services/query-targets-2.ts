import { l, li } from "../../common/utils";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { getBlockTime, wrapListQuerier } from "./utils";
import {
  CommonConfig,
  LiquidationItem,
  RateConfig,
} from "../../common/codegen/LendingPlatform.types";
import {
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

const platformList: PlatformType[] = ["main", "stable"];

// TODO:
// +check it's safe to use utils.cwGetMarketplaceOffers
// +handle both platforms
// +migrate oracle
// +migrate scheduler adapter
// +migrate marketplace adapter
// +remove execution cooldown from scheduller adapter
// +-add query frequency limiter
// can multiple users share same collection offer?
export async function queryTargets(
  chainId: string,
  rpc: string,
  marketplace: string,
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
  bidsPaginationAmount: number,
  offersBatchPaginationAmount: number, // 5
  offersMaxPaginationAmount: number, // 10
  offersMaxCount: number // 20
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

  const collectionList =
    getCollectionsFromLtvAndCollateralList(ltvAndCollateralList);

  const [liquidationBidsMain, liquidationBidsStable] = await Promise.all(
    platformList.map((platformType) =>
      wrapListQuerier(
        lending.pQueryLiquidationBidsByCollectionAddressList,
        platformType,
        bidsPaginationAmount
      )
    )
  );

  // TODO: token querier may not be efficient
  const { royaltyFeeList, tokenOfferList, collectionOfferList } =
    await utils.cwGetMarketplaceData(
      marketplace,
      collectionList,
      blockTime,
      offersBatchPaginationAmount,
      offersMaxPaginationAmount,
      offersMaxCount
    );

  const networkFee = await utils.cwQueryNetworkFee(marketplace);
  const { value: priceRatio } = await oracle.cwQueryPriceRatio();
  const offers = distributeOffers(
    collectionList,
    collateralListMain,
    collateralListStable,
    tokenOfferList,
    collectionOfferList,
    royaltyFeeList,
    networkFee,
    Number(priceRatio),
    blockTime,
    platformList
  );

  // TODO: replace with splitter
  const totalBidsCommon = mergeBidsAndOffers(
    liquidationBidsMain,
    liquidationBidsStable,
    offers
  );

  let totalBidsMain = totalBidsCommon.filter((x) => x.platformType === "main");
  let totalBidsStable = totalBidsCommon.filter(
    (x) => x.platformType === "stable"
  );

  const liquidationTargets = ltvAndCollateralList.reduce(
    (
      acc,
      {
        platformType,
        borrowerAddress,
        ltv,
        loanAmount,
        loanCreationDate,
        accumulatedLoan,
        collateralValue,
        collectionAndCollateralList,
      }
    ) => {
      const isMain = platformType === "main";
      const { main_currency } = isMain ? commonConfigMain : commonConfigStable;
      const { discount_max_rate, borrow_apr } = isMain
        ? rateConfigMain
        : rateConfigStable;
      const ltvMax = isMain ? ltvMaxMain : ltvMaxStable;
      const totalBids = isMain ? totalBidsMain : totalBidsStable;

      const collectionAndBidList = getCollectionAndBidList(
        totalBids,
        collectionAndCollateralList
      );

      const biddedCollateralList = matchCollateralsAndBids(
        main_currency,
        collectionAndCollateralList,
        collectionAndBidList,
        priceList
      );

      // calculate how much tokens and what collection must be liquidated to get proper ltv
      // for worst case (max discount)
      const maxLiquidationValue = calcLiquidationValue(
        collateralValue,
        Number(discount_max_rate),
        ltv,
        ltvMax
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
        ({ totalBidsMain, totalBidsStable } = reduceBids(
          platformType,
          totalBidsMain,
          totalBidsStable,
          liquidationSet
        ));

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
