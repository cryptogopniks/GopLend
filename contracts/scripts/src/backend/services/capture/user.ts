import { l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { getSgQueryHelpers } from "../../../common/account/sg-helpers";
import {
  QueryLiquidationBidsByCollectionAddressListResponseItem,
  TokenItem,
  Addr,
  LiquidationBid,
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

  const getUser = async () => {
    const unbonder = await lending.cwQueryUnbonder(userAddress);
    const borrower = await lending.cwQueryBorrower(userAddress);
    const collateral = await lending.cwQueryCollateralByOwner(userAddress);

    let ltv: string | null = null;
    try {
      ltv = await lending.cwQueryConditionalLtv(userAddress);
    } catch (_) {}

    let availableToBorrow: string | null = null;
    try {
      availableToBorrow = await lending.cwQueryAvailableToBorrow(userAddress);
    } catch (_) {}

    return {
      unbonder,
      borrower,
      collateral,
      ltv,
      availableToBorrow,
    };
  };

  const writeUserInfo = async () => {
    try {
      const { main_currency, bgl_currency } =
        await lending.cwQueryCommonConfig();

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
        await getUser();

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
          if (priceList.is_outdated) throw new Error("Outdated prices!");

          const priceItem = priceList.data.find(
            (y) => y.collection === address
          );
          const collectionPrice = priceItem?.price || "0";

          return {
            collectionAddress: address,
            collectionPrice,
            tokenItemList: collateral[0].token_item_list,
          };
        }
      );

      let bidList = await lending.pQueryLiquidationBidsByCollectionAddressList(
        PAGINATION.BIDS
      );

      // get marketplace offers

      const collectionOfferList = await utils.cwGetMarketMakerData(
        collectionAddresses,
        PAGINATION.OFFERS.BATCH
      );

      const collectionOffers: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
        collectionOfferList.map(([collectionAddress, bids]) => {
          const liquidationBids: LiquidationBid[] = bids.map((x) => {
            const tokenIdList = (
              collateral.find((x) => x.address === collectionAddress)
                ?.collateral || []
            ).flatMap(({ token_item_list }) =>
              token_item_list.map(({ id }) => id)
            );

            return {
              creation_date: blockTime,
              liquidator: "MM",
              token_id_list: tokenIdList,
              amount: x,
              discount: "0",
              bid_type: "collection_offer",
            };
          });

          return {
            collection_address: collectionAddress,
            liquidation_bids: liquidationBids,
          };
        });

      bidList = [...bidList, ...collectionOffers];

      const collateralCost = collateralWithPrices
        .reduce(
          (acc, cur) =>
            acc + Number(cur.collectionPrice) * cur.tokenItemList.length,
          0
        )
        .toString();

      const ownBids = await lending.cwQueryLiquidationBidsByLiquidatorAddress(
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

  await writeUserInfo();
}

main("stars1lze4mhdus8tfh40046sz5r533grvua5rcv0f09");
// main("stars133xakkrfksq39wxy575unve2nyehg5npka2vsp");
