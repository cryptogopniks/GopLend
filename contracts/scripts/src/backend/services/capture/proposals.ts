import { getLast, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { PlatformType } from "../../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { QueryProposalsResponseItem } from "../../../common/codegen/LendingPlatform.types";
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

  const getProposals = async (platformType: PlatformType) => {
    let allItems: QueryProposalsResponseItem[] = [];
    let firstItem: number | undefined = undefined;

    while (firstItem !== 0) {
      const items: QueryProposalsResponseItem[] =
        await lending.cwQueryProposals(
          platformType,
          PAGINATION_QUERY_AMOUNT,
          firstItem,
          true
        );

      firstItem = getLast(items)?.id || 0;
      allItems = [...allItems, ...items];

      l({ firstProposal: firstItem });
    }

    return allItems;
  };

  const platformList: PlatformType[] = ["main", "stable"];
  for (const platform of platformList) {
    l("\n", { platform });
    const proposals = await getProposals(platform);

    // write files
    await writeFile(
      getSnapshotPath(NAME, TYPE, platform, "proposals.json"),
      JSON.stringify(proposals, null, 2),
      {
        encoding: ENCODING,
      }
    );
  }
}

main();
