import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { QueryLiquidatorsResponseItem } from "../../../common/codegen/LendingPlatform.types";
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

  const getLiquidators = async (platformType: PlatformType) => {
    let allItems: QueryLiquidatorsResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryLiquidatorsResponseItem[] =
        await lending.cwQueryLiquidatorList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem,
          true
        );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastLiquidator: lastItem });
    }

    return allItems;
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });
    const liquidators = await getLiquidators(platform);

    // write files
    await writeFile(
      getSnapshotPath(NAME, TYPE, platform, "liquidators.json"),
      JSON.stringify(liquidators),
      {
        encoding: ENCODING,
      }
    );
  }
}

main();
