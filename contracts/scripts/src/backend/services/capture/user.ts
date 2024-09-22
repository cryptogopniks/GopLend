import { l } from "../../../common/utils";
import { calcPrice } from "../math";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { getSgQueryHelpers } from "../../../common/account/sg-helpers";
import {
  LiquidationBid,
  PlatformType,
} from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  QueryLiquidationBidsByCollectionAddressListResponseItem,
  TokenItem,
  Addr,
} from "../../../common/codegen/LendingPlatform.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
} from "../utils";

interface CollateralsResponseItem {
  collectionAddress: Addr;
  collectionPrice: string;
  tokenItemList: TokenItem[];
}

const PAGINATION = {
  PRICES: 200,
  BIDS: 200,
  OFFERS: {
    BATCH: 5,
    COLLECTION_LIMIT: 50,
    TOKEN_LIMIT: 20,
  },
};

async function main(userAddress: string) {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    NAME,
    OPTION: {
      RPC_LIST: [RPC],
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { lending, oracle, utils } = await getCwQueryHelpers(chainId, RPC);
  const { getBalance } = await getSgQueryHelpers(RPC);

  const getUser = async (platformType: PlatformType) => {
    const unbonder = await lending.cwQueryUnbonder(platformType, userAddress);
    const borrower = await lending.cwQueryBorrower(platformType, userAddress);
    const collateral = await lending.cwQueryCollateralByOwner(
      platformType,
      userAddress
    );

    let ltv: string | null = null;
    try {
      ltv = await lending.cwQueryConditionalLtv(platformType, userAddress);
    } catch (_) {}

    let availableToBorrow: string | null = null;
    try {
      availableToBorrow = await lending.cwQueryAvailableToBorrow(
        platformType,
        userAddress
      );
    } catch (_) {}

    return {
      unbonder,
      borrower,
      collateral,
      ltv,
      availableToBorrow,
    };
  };

  const writeUserInfo = async (platformType: PlatformType) => {
    try {
      const isMain = platformType === "main";
      const { is_marketplace_interface_enabled, main_currency, bgl_currency } =
        await lending.cwQueryCommonConfig(platformType);

      const mainToken = main_currency?.token;
      if (!mainToken) throw new Error("mainToken isn't found!");

      const bglToken = bgl_currency?.token;
      if (!bglToken) throw new Error("bglToken isn't found!");

      const mainDenom = "native" in mainToken ? mainToken.native.denom : "";
      const mainTokenBalance = (await getBalance(userAddress, mainDenom))
        .amount;

      const bglDenom = "native" in bglToken ? bglToken.native.denom : "";
      const bglTokenBalance = (await getBalance(userAddress, bglDenom)).amount;

      const { unbonder, borrower, collateral, ltv, availableToBorrow } =
        await getUser(platformType);

      const blockTime = await oracle.cwQueryBlockTime();

      // get collateral collection prices
      const collectionAddresses = collateral.map((x) => x.address);
      const priceList = await oracle.cwQueryPrices(
        PAGINATION.PRICES,
        undefined,
        collectionAddresses
      );

      const collateralWithPrices: CollateralsResponseItem[] = collateral.map(
        ({ address, collateral }) => {
          const priceItem = priceList.find((y) => y.address === address);
          const collectionPrice =
            (isMain
              ? priceItem?.price?.amount
              : priceItem?.usd_price?.amount) || "0";

          return {
            collectionAddress: address,
            collectionPrice,
            tokenItemList: collateral[0].token_item_list,
          };
        }
      );

      let bidList = await lending.pQueryLiquidationBidsByCollectionAddressList(
        platformType,
        PAGINATION.BIDS
      );

      // get marketplace offers
      if (is_marketplace_interface_enabled) {
        const collectionAndTokensList: [string, string[]][] =
          collectionAddresses.map((collection) => {
            const tokenList = collateralWithPrices
              .filter(
                ({ collectionAddress }) => collectionAddress === collection
              )
              .flatMap(({ tokenItemList }) =>
                tokenItemList.map(({ id }) => id)
              );

            return [collection, tokenList];
          });

        const { collectionOfferList, tokenOfferList, discountBaseList } =
          await utils.cwGetMarketplaceData(
            collectionAndTokensList,
            collectionAddresses,
            blockTime,
            PAGINATION.OFFERS.BATCH,
            PAGINATION.OFFERS.COLLECTION_LIMIT,
            PAGINATION.OFFERS.TOKEN_LIMIT
          );

        // query priceRatio directly as MarketplaceData can be empty due to collectionAddresses
        const priceRatioItem = await oracle.cwQueryPriceRatio();
        if (priceRatioItem.is_outdated) {
          throw new Error("priceRatioStable is outdated!");
        }

        const priceRatioStable = Number(priceRatioItem.value);

        const tokenOffers: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
          tokenOfferList.map(([collectionAddress, bids]) => {
            const discountBase = Number(
              discountBaseList.find(
                ([collection]) => collection === collectionAddress
              )?.[1]
            );

            const liquidationBids: LiquidationBid[] = bids.map((x) => {
              const findersFeeRate = Number(x.finders_fee_bps) / 100;

              return {
                creation_date: blockTime,
                liquidator: x.bidder,
                token_id_list: [x.token_id.toString()],
                amount: calcPrice(
                  Number(x.price),
                  isMain ? 1 : priceRatioStable
                ).toString(),
                discount: (discountBase + findersFeeRate).toString(),
                bid_type: "token_offer",
              };
            });

            return {
              collection_address: collectionAddress,
              liquidation_bids: liquidationBids,
            };
          });

        const collectionOffers: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
          collectionOfferList.map(([collectionAddress, bids]) => {
            const discountBase = Number(
              discountBaseList.find(
                ([collection]) => collection === collectionAddress
              )?.[1]
            );

            const liquidationBids: LiquidationBid[] = bids.map((x) => {
              const findersFeeRate = Number(x.finders_fee_bps) / 100;
              const tokenIdList = (
                collateral.find((x) => x.address === collectionAddress)
                  ?.collateral || []
              ).flatMap(({ token_item_list }) =>
                token_item_list.map(({ id }) => id)
              );

              return {
                creation_date: blockTime,
                liquidator: x.bidder,
                token_id_list: tokenIdList,
                amount: calcPrice(
                  Number(x.price),
                  isMain ? 1 : priceRatioStable
                ).toString(),
                discount: (discountBase + findersFeeRate).toString(),
                bid_type: "collection_offer",
              };
            });

            return {
              collection_address: collectionAddress,
              liquidation_bids: liquidationBids,
            };
          });

        bidList = [...bidList, ...tokenOffers, ...collectionOffers];
      }

      const collateralCost = collateralWithPrices
        .reduce(
          (acc, cur) =>
            acc + Number(cur.collectionPrice) * cur.tokenItemList.length,
          0
        )
        .toString();

      const ownBids = await lending.cwQueryLiquidationBidsByLiquidatorAddress(
        platformType,
        userAddress
      );

      let sideBids: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
        [];

      for (const { collection_address, liquidation_bids } of bidList) {
        for (const bid of liquidation_bids) {
          if (
            collateral
              .find((x) => x.address === collection_address)
              ?.collateral.some((y) =>
                y.token_item_list.some((z) => bid.token_id_list.includes(z.id))
              )
          ) {
            const sideBidsByCollection = sideBids.find(
              (x) => x.collection_address === collection_address
            );
            if (!sideBidsByCollection) {
              sideBids.push({
                collection_address,
                liquidation_bids: [],
              });
            }

            sideBids = sideBids.map((x) => {
              if (x.collection_address !== collection_address) {
                return x;
              }

              return {
                collection_address,
                liquidation_bids: [...x.liquidation_bids, bid],
              };
            });
          }
        }
      }

      const userInfo = {
        mainTokenBalance,
        bglTokenBalance,
        unbonder,
        borrower,
        collateralCost,
        collateral: collateralWithPrices,
        ltv,
        availableToBorrow,
        ownBids,
        sideBids,
      };

      // write files
      await writeFile(
        getSnapshotPath(
          NAME,
          TYPE,
          platformType,
          `user-${userAddress.slice(-3)}.json`
        ),
        JSON.stringify(userInfo, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  const platformList: PlatformType[] = ["main", "stable"];
  await Promise.all(
    platformList.map((platformType) => writeUserInfo(platformType))
  );
}

main("stars1lze4mhdus8tfh40046sz5r533grvua5rcv0f09");
// main("stars133xakkrfksq39wxy575unve2nyehg5npka2vsp");
