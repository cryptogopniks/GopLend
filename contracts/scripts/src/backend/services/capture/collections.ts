import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { QueryCollectionsResponseItem } from "../../../common/codegen/LendingPlatform.types";
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

  const getCollections = async (platformType: PlatformType) => {
    let allItems: QueryCollectionsResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryCollectionsResponseItem[] =
        await lending.cwQueryCollectionList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem,
          true
        );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastCollection: lastItem });
    }

    return allItems;
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });
    let collections = await getCollections(platform);

    // sort by name ascending
    collections = collections.sort((a, b) =>
      a.collection.name.localeCompare(b.collection.name)
    );

    // write files
    await writeFile(
      getSnapshotPath(NAME, TYPE, platform, "collections.json"),
      JSON.stringify(collections),
      {
        encoding: ENCODING,
      }
    );
  }
}

main();
