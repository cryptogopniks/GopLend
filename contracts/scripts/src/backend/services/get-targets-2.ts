import { l, li } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { getChainOptionById } from "../../common/config/config-utils";
import { queryTargets } from "./query-targets-2";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getLocalBlockTime,
} from "./utils";
import { calcLtvMax } from "./math";

const PAGINATION = {
  COLLATERAL: 100,
  BORROWERS: 100,
  PRICES: 100,
  BIDS: 100,
};

const platformList: PlatformType[] = ["main", "stable"];

async function main() {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    PREFIX,
    OPTION: {
      RPC_LIST: [RPC],
      DENOM,
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { lending, oracle } = await getCwQueryHelpers(chainId, RPC);

  // initialization part - run it single time on script start
  const [rateConfigMain, rateConfigStable] = await Promise.all(
    platformList.map((platformType) =>
      (async () => await lending.cwQueryRateConfig(platformType))()
    )
  );

  const [commonConfigMain, commonConfigStable] = await Promise.all(
    platformList.map((platformType) =>
      (async () => await lending.cwQueryCommonConfig(platformType))()
    )
  );

  const [ltvMaxMain, ltvMaxStable] = [rateConfigMain, rateConfigStable].map(
    (rateConfig) =>
      calcLtvMax(
        Number(rateConfig.bid_min_rate),
        Number(rateConfig.discount_max_rate),
        Number(rateConfig.discount_min_rate)
      )
  );

  // get blockTimeOffset to fix difference between local and network time
  const blockTime = await oracle.cwQueryBlockTime();
  const localBlockTime = getLocalBlockTime();
  const blockTimeOffset = blockTime - localBlockTime + 10;

  // loop
  while (true) {
    const date = getLocalBlockTime();
    try {
      // get targets
      const targets = await queryTargets(
        chainId,
        RPC,
        rateConfigMain,
        rateConfigStable,
        commonConfigMain,
        commonConfigStable,
        ltvMaxMain,
        ltvMaxStable,
        blockTimeOffset,
        PAGINATION.COLLATERAL,
        PAGINATION.BORROWERS,
        PAGINATION.PRICES,
        PAGINATION.BIDS
      );

      li({ targets });
    } catch (error) {
      l(error, "\n");
    }
  }
}

// TODO: use both platforms
main();
