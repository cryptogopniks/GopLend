import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { QueryBorrowersResponseItem } from "../../../common/codegen/LendingPlatform.types";
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

  const getBorrowers = async (platformType: PlatformType) => {
    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryBorrowersResponseItem[] =
        await lending.cwQueryBorrowerList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem,
          true
        );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastBorrower: lastItem });
    }

    return allItems;
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });
    const borrowers = await getBorrowers(platform);

    // write files
    await writeFile(
      getSnapshotPath(NAME, TYPE, platform, "borrowers.json"),
      JSON.stringify(borrowers),
      {
        encoding: ENCODING,
      }
    );
  }
}

main();
