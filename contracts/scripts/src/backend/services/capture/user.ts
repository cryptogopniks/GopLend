import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import {
  QueryLiquidationBidsByCollectionAddressListResponseItem,
  QueryLiquidationBidsByLiquidatorAddressListResponseItem,
} from "../../../common/codegen/LendingPlatform.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
} from "../utils";

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

  const { lending } = await getCwQueryHelpers(chainId, RPC);

  const getBids = async (platformType: PlatformType) => {
    let allItems: QueryLiquidationBidsByLiquidatorAddressListResponseItem[] =
      [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryLiquidationBidsByLiquidatorAddressListResponseItem[] =
        await lending.cwQueryLiquidationBidsByLiquidatorAddressList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem
        );

      lastItem = getLast(items)?.liquidator_address || "";
      allItems = [...allItems, ...items];

      l({ lastBid: lastItem });
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
    const bidList = await getBids(platform);
    const { unbonder, borrower, collateral, ltv, availableToBorrow } =
      await getUser(platform);

    const ownBids =
      bidList.find((x) => x.liquidator_address === userAddress)
        ?.collection_and_liquidation_bids_list || [];

    let sideBids: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
      [];

    for (const bidListItem of bidList) {
      for (const {
        collection_address,
        liquidation_bids,
      } of bidListItem.collection_and_liquidation_bids_list) {
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
    }

    const userInfo = {
      unbonder,
      borrower,
      collateral,
      ltv,
      availableToBorrow,
      ownBids,
      sideBids,
    };

    // write files
    await writeFile(
      getSnapshotPath(NAME, TYPE, platform, "user.json"),
      JSON.stringify(userInfo, null, 2),
      {
        encoding: ENCODING,
      }
    );
  }
}

main("stars1f37v0rdvrred27tlqqcpkrqpzfv6ddr2a97zzu");
