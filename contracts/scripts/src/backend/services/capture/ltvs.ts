import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { Addr, Decimal } from "../../../common/codegen/LendingPlatform.types";
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

  const writeLtvList = async () => {
    try {
      let allItems: [Addr, Decimal | null][] = [];
      let lastItem: string | undefined = undefined;

      while (lastItem !== "") {
        const items: [Addr, Decimal | null][] = await lending.cwQueryLtvList(
          PAGINATION_QUERY_AMOUNT,
          lastItem
        );

        lastItem = getLast(items)?.[0] || "";
        allItems = [...allItems, ...items];
      }

      let ltvs = allItems;

      // sort by ltv descending
      ltvs = ltvs.sort((a, b) => Number(b[1]) - Number(a[1]));

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "ltvs.json"),
        JSON.stringify(ltvs, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeLtvList();
}

main();
