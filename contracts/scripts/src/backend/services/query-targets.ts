// import { l, li } from "../../common/utils";
// import { getCwQueryHelpers } from "../../common/account/cw-helpers";
// import { getBlockTime, wrapListQuerier } from "./utils";
// import {
//   CommonConfig,
//   LiquidationItem,
//   RateConfig,
// } from "../../common/codegen/LendingPlatform.types";
// import {
//   aggregateTokensByCollection,
//   calcLiquidationSet,
//   calcLiquidationValue,
//   distributeOffers,
//   getCollectionAndBidList,
//   getCollectionsFromLtvAndCollateralList,
//   getTargetBorrowers,
//   matchCollateralsAndBids,
//   mergeBidsAndOffers,
//   reduceBids,
// } from "./math";

// export async function queryTargets(
//   chainId: string,
//   rpc: string,
//   rateConfig: RateConfig,
//   commonConfig: CommonConfig,
//   ltvMax: number,
//   ltvMaxFraction: number,
//   blockTimeOffset: number, // several seconds to consider query delays
//   collateralPaginationAmount: number,
//   borrowersPaginationAmount: number,
//   pricesPaginationAmount: number,
//   bidsPaginationAmount: number,
//   offersBatchPaginationAmount: number, // 5
//   collectionOffersPaginationAmount: number, // 50
//   tokenOffersPaginationAmount: number // 20
// ): Promise<LiquidationItem[]> {
//   const { lending, oracle, utils } = await getCwQueryHelpers(chainId, rpc);

//   const collateralList = await lending.pQueryCollateralList(
//     collateralPaginationAmount
//   );
//   const borrowerList = await lending.pQueryBorrowerList(
//     borrowersPaginationAmount
//   );
//   const priceList = await oracle.pQueryPrices(pricesPaginationAmount);
//   const blockTime = getBlockTime(blockTimeOffset);

//   // get borrowers with ltv > ltvMax, sort by ltv descending
//   const ltvAndCollateralList = getTargetBorrowers(
//     borrowerList,
//     collateralList,
//     priceList,
//     rateConfig,
//     commonConfig,
//     ltvMax,
//     blockTime
//   ).sort(({ ltv: ltvA }, { ltv: ltvB }) => ltvB - ltvA);

//   if (!ltvAndCollateralList.length) {
//     return [];
//   }

//   const collectionList =
//     getCollectionsFromLtvAndCollateralList(ltvAndCollateralList);

//   const liquidationBids =
//     await lending.pQueryLiquidationBidsByCollectionAddressList(
//       bidsPaginationAmount
//     );

//   const collectionAndTokensList = aggregateTokensByCollection(
//     ltvAndCollateralList,
//     collectionList
//   );

//   const collectionOfferList = await utils.cwGetMarketplaceData(
//     collectionList,
//     offersBatchPaginationAmount
//   );

//   const offers = distributeOffers(
//     collectionList,
//     collateralList,
//     collectionOfferList,
//     blockTime
//   );

//   let totalBids = mergeBidsAndOffers(liquidationBids, offers);

//   const liquidationTargets: [PlatformType, LiquidationItem][] =
//     ltvAndCollateralList.reduce(
//       (
//         acc,
//         {
//           platformType,
//           borrowerAddress,
//           ltv,
//           loanAmount,
//           loanCreationDate,
//           accumulatedLoan,
//           collateralValue,
//           collectionAndCollateralList,
//         }
//       ) => {
//         const isMain = platformType === "main";
//         const { main_currency } = isMain
//           ? commonConfigMain
//           : commonConfigStable;
//         const { discount_max_rate, borrow_apr } = isMain
//           ? rateConfigMain
//           : rateConfigStable;
//         const ltvMax = isMain ? ltvMaxMain : ltvMaxStable;
//         const totalBids = isMain ? totalBidsMain : totalBidsStable;

//         const collectionAndBidList = getCollectionAndBidList(
//           totalBids,
//           collectionAndCollateralList
//         );

//         const biddedCollateralList = matchCollateralsAndBids(
//           main_currency,
//           collectionAndCollateralList,
//           collectionAndBidList,
//           priceList
//         );

//         // calculate how much tokens and what collection must be liquidated to get proper ltv
//         // for worst case (max discount)
//         const maxLiquidationValue = calcLiquidationValue(
//           collateralValue,
//           Number(discount_max_rate),
//           ltv,
//           ltvMax,
//           ltvMaxFraction
//         );

//         const liquidationSet = calcLiquidationSet(
//           biddedCollateralList,
//           maxLiquidationValue,
//           ltvMax,
//           Number(borrow_apr),
//           accumulatedLoan,
//           loanAmount,
//           collateralValue,
//           blockTime,
//           loanCreationDate
//         );

//         if (liquidationSet.length) {
//           // remove used bids
//           ({ totalBidsMain, totalBidsStable } = reduceBids(
//             platformType,
//             totalBidsMain,
//             totalBidsStable,
//             liquidationSet
//           ));

//           acc.push([
//             platformType,
//             {
//               borrower: borrowerAddress,
//               liquidation_set: liquidationSet,
//             },
//           ]);
//         } else {
//           l(
//             `⚠️⚠️⚠️ Warning: No (proper) bids/offers to liquidate ${borrowerAddress}`
//           );
//         }

//         return acc;
//       },
//       [] as [PlatformType, LiquidationItem][]
//     );

//   return liquidationTargets;
// }
