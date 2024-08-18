import { l, li } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { getChainOptionById } from "../../common/config/config-utils";
import { queryTargets } from "./query-targets-2";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import { ENCODING, PATH_TO_CONFIG_JSON, parseStoreArgs } from "./utils";
import { calcLtvMax } from "./math";

// const PAGINATION_QUERY_AMOUNT = 100;

async function main(platformType: PlatformType) {
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

  const { lending } = await getCwQueryHelpers(chainId, RPC);

  // TODO: single query
  const [rateConfig, commonConfig] = await Promise.all([
    lending.cwQueryRateConfig(platformType, true),
    lending.cwQueryCommonConfig(platformType, true),
  ]);

  const ltvMax = calcLtvMax(
    Number(rateConfig.bid_min_rate),
    Number(rateConfig.discount_max_rate),
    Number(rateConfig.discount_min_rate)
  );

  // get targets
  // const targets = await queryTargets(platformType, borrowerAddressList);
  // li({ targets });
}

// TODO: use both platforms
main("main");
