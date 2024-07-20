import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { getSgQueryHelpers } from "../../../common/account/sg-helpers";
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

const PAGINATION_QUERY_AMOUNT = 200;

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

  const { adapter, lending, oracle } = await getCwQueryHelpers(chainId, RPC);
  const { getBalance } = await getSgQueryHelpers(RPC);

  const getBids = async (platformType: PlatformType) => {
    let allItems: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
      [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
        await lending.cwQueryLiquidationBidsByCollectionAddressList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem
        );

      lastItem = getLast(items)?.collection_address || "";
      allItems = [...allItems, ...items];

      l({ lastBid: lastItem });
    }

    return allItems;
  };

  const getOffers = async (
    platformType: PlatformType,
    is_marketplace_interface_enabled: boolean
  ) => {
    if (!is_marketplace_interface_enabled) {
      return [];
    }

    let allItems: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
      [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
        await adapter.marketplace.stargaze.cwQueryOffersByCollectionAddressList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem
        );

      lastItem = getLast(items)?.collection_address || "";
      allItems = [...allItems, ...items];

      l({ lastOffer: lastItem });
    }

    return allItems;
  };

  const getUser = async (platformType: PlatformType) => {
    const unbonder = await lending.cwQueryUnbonder(platformType, userAddress);
    const borrower = await lending.cwQueryBorrower(platformType, userAddress);
    const collateral = await lending.cwQueryCollateralByOwner(
      platformType,
      userAddress
    );
    const ltv = await lending.cwQueryConditionalLtv(platformType, userAddress);
    const availableToBorrow = await lending.cwQueryAvailableToBorrow(
      platformType,
      userAddress
    );

    return {
      unbonder,
      borrower,
      collateral,
      ltv,
      availableToBorrow,
    };
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });

    const { is_marketplace_interface_enabled, bgl_currency } =
      await lending.cwQueryCommonConfig(platform);

    const bglToken = bgl_currency?.token;
    if (!bglToken) throw new Error("bglToken isn't found!");

    const bglDenom = "native" in bglToken ? bglToken.native.denom : "";
    const bglTokenBalance = (await getBalance(userAddress, bglDenom)).amount;

    let bidList = await getBids(platform);
    const offerList = await getOffers(
      platform,
      is_marketplace_interface_enabled
    );
    bidList = [...bidList, ...offerList];

    const { unbonder, borrower, collateral, ltv, availableToBorrow } =
      await getUser(platform);

    // get collateral collection prices
    const collectionAddresses = collateral.map((x) => x.address);
    const priceList = await oracle.cwQueryPrices(
      PAGINATION_QUERY_AMOUNT,
      undefined,
      collectionAddresses
    );

    const collateralWithPrices: CollateralsResponseItem[] = collateral.map(
      ({ address, collateral }) => {
        const priceItem = priceList.find((y) => y.address === address);
        const collectionPrice =
          (platform === "main"
            ? priceItem?.price?.amount
            : priceItem?.usd_price?.amount) || "0";

        return {
          collectionAddress: address,
          collectionPrice,
          tokenItemList: collateral[0].token_item_list,
        };
      }
    );

    const collateralCost = collateralWithPrices
      .reduce(
        (acc, cur) =>
          acc + Number(cur.collectionPrice) * cur.tokenItemList.length,
        0
      )
      .toString();

    const ownBids = await lending.cwQueryLiquidationBidsByLiquidatorAddress(
      platform,
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
        platform,
        `user-${userAddress.slice(-3)}.json`
      ),
      JSON.stringify(userInfo, null, 2),
      {
        encoding: ENCODING,
      }
    );
  }
}

main("stars1gjqnuhv52pd2a7ets2vhw9w9qa9knyhyzrpx49");
