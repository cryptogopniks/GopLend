// import { getSigner } from "../account/signer";
// import { l, wait, Request } from "../../common/utils";
// import { readFile } from "fs/promises";
// import { getChainOptionById } from "../../common/config/config-utils";
// import { ChainConfig } from "../../common/interfaces";
// import {
//   getCwExecHelpers,
//   getCwQueryHelpers,
// } from "../../common/account/cw-helpers";
// import {
//   CollectionPriceResponse,
//   getOffersExtendedDown,
//   getOffersExtendedUp,
//   splitOffers,
// } from "./math";
// import {
//   ENCODING,
//   PATH_TO_CONFIG_JSON,
//   getWallets,
//   parseStoreArgs,
// } from "./utils";

// const DELAY = 6_000;
// const MAX_INACTIVE_OFFERS = 5;
// const FLOOR_PRICE_BOUNDARY = {
//   UPPER: 0.93,
//   LOWER: 0.8,
// };
// const ALLOWED_OFFERS_BOUNDARY = {
//   UPPER: 0.8325,
//   LOWER: 0.8,
// };

// async function main(isInfiniteLoop: boolean) {
//   const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
//     encoding: ENCODING,
//   });
//   const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
//   const { chainId } = parseStoreArgs();
//   const {
//     NAME,
//     PREFIX,
//     OPTION: {
//       RPC_LIST: [RPC],
//       DENOM,
//       GAS_PRICE_AMOUNT,
//       TYPE,
//     },
//   } = getChainOptionById(CHAIN_CONFIG, chainId);

//   const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
//   const testWallets = await getWallets(TYPE);
//   const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

//   const { marketMaker } = await getCwQueryHelpers(chainId, RPC);
//   const h = await getCwExecHelpers(chainId, RPC, owner, signer);
//   const req = new Request();

//   // [collection, CollectionPriceResponse][]
//   let collectionPriceResponseList: [string, CollectionPriceResponse][] = [];

//   // loop
//   while (true) {
//     try {
//       // TODO: query collectionPriceResponseList from DB
//       // collectionPriceResponseList = await queryCollectionPriceResponseList()

//       for (const [collection, { floorPrice }] of collectionPriceResponseList) {
//         const liquidity = await marketMaker.hub.cwQueryLiquidity(collection);
//         const amountUndistributed = Number(liquidity.amount_undistributed);
//         const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
//         const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
//         const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
//           lowerFloorPrice,
//           upperFloorPrice,
//           liquidity.price_and_outpost_list
//         );

//         // extend offers up
//         if (tooSmallOffers.length >= MAX_INACTIVE_OFFERS) {
//           const {
//             fromToPriceAndOutpostList,
//             isNoLiquidity,
//             isRequiredMoreLiquidity,
//           } = getOffersExtendedUp(
//             ALLOWED_OFFERS_BOUNDARY.UPPER,
//             amountUndistributed,
//             floorPrice,
//             lowerFloorPrice,
//             regularOffers,
//             tooSmallOffers
//           );

//           if (isNoLiquidity) {
//             l(`⚠️⚠️⚠️ Warning: No liquidity for collection ${collection}`);
//             continue;
//           }

//           if (isRequiredMoreLiquidity) {
//             l(`⚠️⚠️⚠️ Warning: Add liquidity for collection ${collection}`);
//           }

//           await h.marketMaker.hub.cwUpdateOffers(
//             collection,
//             fromToPriceAndOutpostList,
//             gasPrice
//           );
//           continue;
//         }

//         // extend offers down
//         if (tooBigOffers.length >= MAX_INACTIVE_OFFERS) {
//           const { fromToPriceAndOutpostList, isNoLiquidity } =
//             getOffersExtendedDown(
//               ALLOWED_OFFERS_BOUNDARY.LOWER,
//               amountUndistributed,
//               floorPrice,
//               upperFloorPrice,
//               regularOffers,
//               tooBigOffers
//             );

//           if (isNoLiquidity) {
//             l(`⚠️⚠️⚠️ Warning: No liquidity for collection ${collection}`);
//             continue;
//           }

//           await h.marketMaker.hub.cwUpdateOffers(
//             collection,
//             fromToPriceAndOutpostList,
//             gasPrice
//           );
//         }
//       }
//     } catch (error) {
//       l(error, "\n");
//     }
//     await wait(DELAY);

//     if (!isInfiniteLoop) break;
//   }
// }

// // TODO: set true for production
// main(true);
