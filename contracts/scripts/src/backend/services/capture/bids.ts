import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { QueryLiquidationBidsByLiquidatorAddressListResponseItem } from "../../../common/codegen/LendingPlatform.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
} from "../utils";

const PAGINATION_QUERY_AMOUNT = 200;

async function main() {
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

  const writeBids = async () => {
    try {
      let allItems: QueryLiquidationBidsByLiquidatorAddressListResponseItem[] =
        [];
      let lastItem: string | undefined = undefined;

      while (lastItem !== "") {
        const items: QueryLiquidationBidsByLiquidatorAddressListResponseItem[] =
          await lending.cwQueryLiquidationBidsByLiquidatorAddressList(
            PAGINATION_QUERY_AMOUNT,
            lastItem
          );

        lastItem = getLast(items)?.liquidator_address || "";
        allItems = [...allItems, ...items];
      }

      let bids = allItems;

      // sort by amount descending
      bids = bids.sort(
        (a, b) =>
          b.collection_and_liquidation_bids_list.reduce(
            (acc1, cur1) =>
              acc1 +
              cur1.liquidation_bids.reduce(
                (acc2, cur2) => acc2 + Number(cur2.amount),
                0
              ),
            0
          ) -
          a.collection_and_liquidation_bids_list.reduce(
            (acc1, cur1) =>
              acc1 +
              cur1.liquidation_bids.reduce(
                (acc2, cur2) => acc2 + Number(cur2.amount),
                0
              ),
            0
          )
      );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "bids.json"),
        JSON.stringify(bids, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeBids();
}

main();
