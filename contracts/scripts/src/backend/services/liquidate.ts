import { getMultipleSigners, SignerData } from "../account/signer";
import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { queryTargets } from "./query-targets";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { Route } from "../../common/codegen/AdapterDexStargaze.types";
import { calcLtvMax } from "./math";
import {
  getChainOptionById,
  getContractByLabel,
} from "../../common/config/config-utils";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import {
  CommonConfig,
  LiquidationItem,
  RateConfig,
} from "../../common/codegen/LendingPlatform.types";
import {
  getCwClient,
  signAndBroadcastWrapper,
} from "../../common/account/clients";
import {
  MsgExecuteContractEncodeObject,
  SigningCosmWasmClient,
} from "@cosmjs/cosmwasm-stargate";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
  specifyTimeout,
  getLocalBlockTime,
} from "./utils";

const SIGNERS_AMOUNT = 5;
const PUSH_TIMEOUT_S = 10;

const LTV_MAX_FRACTION = 0.9;
const BLOCK_TIME_MARGIN = 10; // delay in seconds to consider paginated queries time difference

const PAGINATION = {
  COLLATERAL: 100,
  BORROWERS: 100,
  PRICES: 100,
  BIDS: 100,
  OFFERS: {
    BATCH: 5,
    COLLECTION_LIMIT: 50,
    TOKEN_LIMIT: 20,
  },
  TARGETS_PER_CYCLE: 9,
};

const platformList: PlatformType[] = ["main", "stable"];

// returns list of functions (pushFn) initialized with separate account client
// Each pushFn allows to send tx with multiple Push msgs
// while msg count is equal amount of unique routes and for each msg
// liquidation items are aggregated by route
async function getPushListFn(
  signerList: SignerData[],
  rpc: string,
  contractAddress: string,
  gasPrice: string
) {
  let pushListFn = [];

  for (const { signer, owner } of signerList) {
    const cwClient = await getCwClient(rpc, owner, signer);
    if (!cwClient) throw new Error("cwClient is not found!");

    const signingClient = cwClient.client as SigningCosmWasmClient;
    const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

    const pushFn = async (
      routeAndliquidationItemList: [Route, LiquidationItem][]
    ) => {
      const routeList = Array.from(
        new Set(routeAndliquidationItemList.map(([route]) => route))
      );

      const msgList = routeList.map((route) => {
        const targets = routeAndliquidationItemList.reduce(
          (acc, [currentRoute, liquidationItem]) => {
            if (currentRoute === route) acc.push(liquidationItem);
            return acc;
          },
          [] as LiquidationItem[]
        );

        const msg: MsgExecuteContractEncodeObject = {
          typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
          value: MsgExecuteContract.fromPartial({
            sender: owner,
            contract: contractAddress,
            msg: toUtf8(
              JSON.stringify({
                push: {
                  route,
                  targets,
                },
              })
            ),
            funds: [],
          }),
        };

        return msg;
      });

      await _signAndBroadcast(msgList, gasPrice);
    };

    pushListFn.push(pushFn);
  }

  return pushListFn;
}

function groupTargets(targets: [PlatformType, LiquidationItem][]) {
  targets = targets.sort(
    ([_a, a], [_b, b]) => b.liquidation_set.length - a.liquidation_set.length
  );

  let pairs: [PlatformType, LiquidationItem][][] = [];

  if (targets.length % 2) {
    pairs.push([targets[0]]);
    targets = targets.slice(1);
  }

  const halfLength = targets.length / 2;
  for (let i = 0; i < halfLength; i++) {
    pairs.push([targets[i], targets[halfLength - i]]);
  }

  return pairs;
}

async function liquidate(
  chainId: string,
  rpc: string,
  rateConfigMain: RateConfig,
  rateConfigStable: RateConfig,
  commonConfigMain: CommonConfig,
  commonConfigStable: CommonConfig,
  ltvMaxMain: number,
  ltvMaxStable: number,
  ltvMaxFraction: number,
  blockTimeOffset: number, // several seconds to consider query delays
  pushListFn: ((
    routeAndliquidationItemList: [Route, LiquidationItem][]
  ) => Promise<void>)[]
) {
  try {
    // get targets
    let targets = await queryTargets(
      chainId,
      rpc,
      rateConfigMain,
      rateConfigStable,
      commonConfigMain,
      commonConfigStable,
      ltvMaxMain,
      ltvMaxStable,
      ltvMaxFraction,
      blockTimeOffset,
      PAGINATION.COLLATERAL,
      PAGINATION.BORROWERS,
      PAGINATION.PRICES,
      PAGINATION.BIDS,
      PAGINATION.OFFERS.BATCH,
      PAGINATION.OFFERS.COLLECTION_LIMIT,
      PAGINATION.OFFERS.TOKEN_LIMIT
    );

    // limit targets amount by 9 to not affect too much on collection price
    // and be able to group targets by 2 between 5 accounts (5 + 4 targets is 3 + 2 pairs)
    targets = targets.slice(0, PAGINATION.TARGETS_PER_CYCLE);

    // it's better to aggregate targets by platformType/route to avoid multi message txs
    let targetsMain = targets.filter(([x]) => x === "main");
    let targetsStable = targets.filter(([x]) => x === "stable");

    // We need to distribute targets betwen pushFn functions to balance load
    // Create pairs of targets by combining the target with the maximum number of tokens
    // and the target with the minimum number of tokens
    const targetsGroups = [
      ...groupTargets(targetsMain),
      ...groupTargets(targetsStable),
    ];

    // liquidate using multiple accounts (user per acc)
    const promiseList = targetsGroups.map((groups, i) =>
      (async () => {
        const routeAndliquidationItemList: [Route, LiquidationItem][] =
          groups.map(([platformType, liquidationItem]) => {
            const route: Route = platformType === "main" ? "empty" : "osmosis";
            return [route, liquidationItem];
          });

        try {
          await specifyTimeout(
            pushListFn[i](routeAndliquidationItemList),
            PUSH_TIMEOUT_S * 1e3
          );
        } catch (_) {}
      })()
    );

    await Promise.all(promiseList);
  } catch (error) {
    l(error);
  }
}

async function main() {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    NAME,
    PREFIX,
    OPTION: {
      RPC_LIST: [RPC],
      DENOM,
      GAS_PRICE_AMOUNT,
      TYPE,
      CONTRACTS,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const ADAPTER_SCHEDULER_CONTRACT = getContractByLabel(
    CONTRACTS,
    "adapter_scheduler"
  );

  const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
  const testWallets = await getWallets(TYPE);
  const signerList = await getMultipleSigners(
    PREFIX,
    testWallets.SEED_ADMIN,
    SIGNERS_AMOUNT
  );
  const [{ signer, owner }] = signerList;

  const { lending, oracle } = await getCwQueryHelpers(chainId, RPC);
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);

  const pushListFn = await getPushListFn(
    signerList,
    RPC,
    ADAPTER_SCHEDULER_CONTRACT.ADDRESS,
    gasPrice
  );

  let route: Route = "empty";

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
  const blockTimeOffset = blockTime - localBlockTime + BLOCK_TIME_MARGIN;

  // run liquidation in infinite loop
  while (true) {
    await liquidate(
      chainId,
      RPC,
      rateConfigMain,
      rateConfigStable,
      commonConfigMain,
      commonConfigStable,
      ltvMaxMain,
      ltvMaxStable,
      LTV_MAX_FRACTION,
      blockTimeOffset,
      pushListFn
    );

    // switch route
    if (route === "empty") {
      // TODO: query network/dex status and choose route
      route = "osmosis";
    } else {
      route = "empty";
    }
  }
}

main();
