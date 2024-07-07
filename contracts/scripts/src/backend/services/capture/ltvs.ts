import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
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

  const getLtvList = async (platformType: PlatformType) => {
    let allItems: [Addr, Decimal][] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: [Addr, Decimal][] = await lending.cwQueryLtvList(
        platformType,
        PAGINATION_QUERY_AMOUNT,
        lastItem,
        true
      );

      lastItem = getLast(items)?.[0] || "";
      allItems = [...allItems, ...items];

      l({ lastLtv: lastItem });
    }

    return allItems;
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });
    const ltvs = await getLtvList(platform);

    // write files
    await writeFile(
      getSnapshotPath(NAME, TYPE, platform, "ltvs.json"),
      JSON.stringify(ltvs),
      {
        encoding: ENCODING,
      }
    );
  }
}

main();
