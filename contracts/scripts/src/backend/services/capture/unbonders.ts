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

  const writeUnbonders = async () => {
    try {
      // sort by returned amount descending
      const unbonders = (
        await lending.pQueryUnbonderList(PAGINATION_QUERY_AMOUNT)
      ).sort(
        (a, b) =>
          b.unbonder.returned_and_unbonded.reduce(
            (acc, cur) => acc + Number(cur[0]),
            0
          ) -
          a.unbonder.returned_and_unbonded.reduce(
            (acc, cur) => acc + Number(cur[0]),
            0
          )
      );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "unbonders.json"),
        JSON.stringify(unbonders, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeUnbonders();
}

main();
