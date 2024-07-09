import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { QueryUnbondersResponseItem } from "../../../common/codegen/LendingPlatform.types";
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

  const getUnbonders = async (platformType: PlatformType) => {
    let allItems: QueryUnbondersResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryUnbondersResponseItem[] =
        await lending.cwQueryUnbonderList(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          lastItem
        );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastUnbonder: lastItem });
    }

    return allItems;
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });
    let unbonders = await getUnbonders(platform);

    // sort by returned amount descending
    unbonders = unbonders.sort(
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
      getSnapshotPath(NAME, TYPE, platform, "unbonders.json"),
      JSON.stringify(unbonders, null, 2),
      {
        encoding: ENCODING,
      }
    );
  }
}

main();
