import { l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
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

  const writeCollections = async () => {
    try {
      // sort by name ascending
      const collections = (
        await lending.pQueryCollectionList(PAGINATION_QUERY_AMOUNT)
      ).sort((a, b) => a.collection.name.localeCompare(b.collection.name));

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "collections.json"),
        JSON.stringify(collections, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeCollections();
}

main();
