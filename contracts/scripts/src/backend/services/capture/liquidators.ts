import { l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
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

  const writeLiquidators = async (platformType: PlatformType) => {
    try {
      // sort by collection amount descending
      const liquidators = (
        await lending.pQueryLiquidatorList(
          platformType,
          PAGINATION_QUERY_AMOUNT
        )
      ).sort(
        (a, b) =>
          b.liquidator.collection_addresses.length -
          a.liquidator.collection_addresses.length
      );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, platformType, "liquidators.json"),
        JSON.stringify(liquidators, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  const platformList: PlatformType[] = ["main", "stable"];
  await Promise.all(
    platformList.map((platformType) => writeLiquidators(platformType))
  );
}

main();
