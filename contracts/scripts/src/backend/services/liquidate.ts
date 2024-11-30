import { getMultipleSigners, SignerData } from "../account/signer";
import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { queryTargets } from "./query-targets";
import { calcLtvMax } from "./math";
import { ExecuteMsg } from "../../common/codegen/Scheduler.types";
import {
  getChainOptionById,
  getContractByLabel,
} from "../../common/config/config-utils";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import {
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
  },
  TARGETS_PER_CYCLE: 9,
};

// returns list of functions (pushFn) initialized with separate account client
// Each pushFn allows to send tx with multiple Push msgs
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

    const pushFn = async (liquidationItemList: LiquidationItem[]) => {
      const pushMsg: ExecuteMsg = {
        push: { targets: liquidationItemList },
      };

      const msg: MsgExecuteContractEncodeObject = {
        typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
        value: MsgExecuteContract.fromPartial({
          sender: owner,
          contract: contractAddress,
          msg: toUtf8(JSON.stringify(pushMsg)),
          funds: [],
        }),
      };

      await _signAndBroadcast([msg], gasPrice);
    };

    pushListFn.push(pushFn);
  }

  return pushListFn;
}

function groupTargets(targets: LiquidationItem[]) {
  targets = targets.sort(
    (a, b) => b.liquidation_set.length - a.liquidation_set.length
  );

  let pairs: LiquidationItem[][] = [];

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
  rateConfig: RateConfig,
  ltvMax: number,
  ltvMaxFraction: number,
  blockTimeOffset: number, // several seconds to consider query delays
  pushListFn: ((
    routeAndliquidationItemList: LiquidationItem[]
  ) => Promise<void>)[]
) {
  try {
    // get targets
    let targets = await queryTargets(
      chainId,
      rpc,
      rateConfig,
      ltvMax,
      ltvMaxFraction,
      blockTimeOffset,
      PAGINATION.COLLATERAL,
      PAGINATION.BORROWERS,
      PAGINATION.PRICES,
      PAGINATION.BIDS,
      PAGINATION.OFFERS.BATCH
    );

    // limit targets amount by 9 to not affect too much on collection price
    // and be able to group targets by 2 between 5 accounts (5 + 4 targets is 3 + 2 pairs)
    targets = targets.slice(0, PAGINATION.TARGETS_PER_CYCLE);

    // We need to distribute targets betwen pushFn functions to balance load
    // Create pairs of targets by combining the target with the maximum number of tokens
    // and the target with the minimum number of tokens
    const targetsGroups = groupTargets(targets);

    // liquidate using multiple accounts (user per acc)
    const promiseList = targetsGroups.map((groups, i) =>
      (async () => {
        try {
          await specifyTimeout(pushListFn[i](groups), PUSH_TIMEOUT_S * 1e3);
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

  const SCHEDULER_CONTRACT = getContractByLabel(CONTRACTS, "scheduler");

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
    SCHEDULER_CONTRACT.ADDRESS,
    gasPrice
  );

  // initialization part - run it single time on script start
  const rateConfig = await lending.cwQueryRateConfig();

  const ltvMax = calcLtvMax(
    Number(rateConfig.bid_min_rate),
    Number(rateConfig.discount_max_rate),
    Number(rateConfig.discount_min_rate)
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
      rateConfig,
      ltvMax,
      LTV_MAX_FRACTION,
      blockTimeOffset,
      pushListFn
    );
  }
}

main();
