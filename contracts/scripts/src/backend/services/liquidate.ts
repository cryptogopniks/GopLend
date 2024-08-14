import { getMultipleSigners, SignerData } from "../account/signer";
import { getLast, l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { queryTargets } from "./query-targets";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { Route } from "../../common/codegen/AdapterDexStargaze.types";
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
  QueryBorrowersResponseItem,
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
} from "./utils";

const PAGINATION_QUERY_AMOUNT = 100;
const SIGNERS_AMOUNT = 50;
const PUSH_TIMEOUT_S = 10;

// TODO: Sort targets by ltv
// TODO: Liquidate multiple users per account if tokens amount is low - must be found via e2e
// TODO: Remove execution cooldown - check logs to decide if we need to send txs
// TODO: sort accounts by balance
// TODO: remove accounts with failed txs
// TODO: use multiple RPC
async function getPushFnList(
  signerList: SignerData[],
  rpc: string,
  contractAddress: string,
  gasPrice: string
) {
  let pushFnList = [];

  for (const { signer, owner } of signerList) {
    const cwClient = await getCwClient(rpc, owner, signer);
    if (!cwClient) throw new Error("cwClient is not found!");

    const signingClient = cwClient.client as SigningCosmWasmClient;
    const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

    const pushFn = async (
      route: Route,
      liquidationItemList: LiquidationItem[]
    ) => {
      const msg: MsgExecuteContractEncodeObject = {
        typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
        value: MsgExecuteContract.fromPartial({
          sender: owner,
          contract: contractAddress,
          msg: toUtf8(
            JSON.stringify({
              push: {
                route,
                targets: liquidationItemList,
              },
            })
          ),
          funds: [],
        }),
      };

      await _signAndBroadcast([msg], gasPrice);
    };

    pushFnList.push(pushFn);
  }

  return pushFnList;
}

async function liquidate(
  route: Route,
  queryBorrowerListFn: (
    platformType: PlatformType,
    paginationQueryAmount: number,
    lastItem?: string
  ) => Promise<QueryBorrowersResponseItem[]>,
  pushFnList: ((
    route: Route,
    liquidationItemList: LiquidationItem[]
  ) => Promise<void>)[]
) {
  try {
    const platformType: PlatformType = route === "empty" ? "main" : "stable";

    // query all borrowers
    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryBorrowersResponseItem[] = await queryBorrowerListFn(
        platformType,
        PAGINATION_QUERY_AMOUNT,
        lastItem
      );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastBorrower: lastItem });
    }

    const borrowerAddressList = allItems.map((x) => x.address);

    // get targets
    const targets = await queryTargets(platformType, borrowerAddressList);

    // liquidate using multiple accounts (user per acc)
    const promiseList = targets.map((liquidationItem, i) =>
      (async () => {
        try {
          await specifyTimeout(
            pushFnList[i](route, [liquidationItem]),
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

  const { lending } = await getCwQueryHelpers(chainId, RPC);
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);

  const name = NAME as "kujira" | "stargaze";

  const cwQueryBorrowerList = async (
    platformType: PlatformType,
    paginationQueryAmount: number,
    lastItem?: string
  ): Promise<QueryBorrowersResponseItem[]> => {
    return await lending.cwQueryBorrowerList(
      platformType,
      paginationQueryAmount,
      lastItem,
      true
    );
  };

  const pushFnList = await getPushFnList(
    signerList,
    RPC,
    ADAPTER_SCHEDULER_CONTRACT.ADDRESS,
    gasPrice
  );

  let route: Route = "empty";

  // run liquidation in infinite loop
  while (true) {
    await liquidate(route, cwQueryBorrowerList, pushFnList);

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
