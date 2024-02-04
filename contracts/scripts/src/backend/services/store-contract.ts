import { l } from "../../common/utils";
import { rootPath } from "../envs";
import { calculateFee, Event } from "@cosmjs/stargate";
import { getChainOptionById } from "../../common/config/config-utils";
import { gzip } from "pako";
import { readFile, writeFile } from "fs/promises";
import { getCwClient } from "../../common/account/clients";
import { getSigner } from "../account/signer";
import { Wasm, ContractInfo, ChainConfig } from "../../common/interfaces";
import { MsgStoreCode } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import {
  SigningCosmWasmClient,
  MsgStoreCodeEncodeObject,
} from "@cosmjs/cosmwasm-stargate";
import {
  parseStoreArgs,
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
} from "./utils";

function parseCodeIdListLegacy(rawLog: string): number[] {
  const regex = /"code_id","value":"(\d+)"/g;

  return (rawLog.match(regex) || []).map(
    (x) => +x.split(":")[1].replace(/"/g, "")
  );
}

function parseCodeIdList(events: readonly Event[]): number[] {
  return events
    .filter((x) => x.type === "store_code")
    .map((x) => +(x.attributes.find((y) => y.key === "code_id")?.value || ""));
}

async function main() {
  try {
    const configJsonStr: string = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    let configJson: ChainConfig = JSON.parse(configJsonStr);

    const { chainId, wasmList } = parseStoreArgs();
    const {
      PREFIX,
      OPTION: {
        DENOM,
        RPC_LIST: [RPC],
        GAS_PRICE_AMOUNT,
        STORE_CODE_GAS_MULTIPLIER,
        CONTRACTS,
        TYPE,
      },
    } = getChainOptionById(configJson, chainId);

    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);
    const cwClient = await getCwClient(RPC, owner, signer);
    if (!cwClient) throw new Error("cwClient is not found!");

    const signingClient = cwClient.client as SigningCosmWasmClient;

    let byteLengthSum = 0;
    let contractConfigAndStoreCodeMsgList: [
      ContractInfo,
      MsgStoreCodeEncodeObject
    ][] = [];

    for (const CONTRACT of CONTRACTS) {
      if (!wasmList.includes(CONTRACT.WASM as Wasm)) continue;

      const wasmBinary = await readFile(
        rootPath(`../artifacts/${CONTRACT.WASM}`)
      );
      const compressed = gzip(wasmBinary, { level: 9 });

      byteLengthSum += compressed.byteLength;

      const storeCodeMsg: MsgStoreCodeEncodeObject = {
        typeUrl: "/cosmwasm.wasm.v1.MsgStoreCode",
        value: MsgStoreCode.fromPartial({
          sender: owner,
          wasmByteCode: compressed,
        }),
      };

      contractConfigAndStoreCodeMsgList.push([CONTRACT, storeCodeMsg]);
    }

    const gasWantedCalc = Math.ceil(STORE_CODE_GAS_MULTIPLIER * byteLengthSum);
    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const fee = calculateFee(gasWantedCalc, gasPrice);

    let codeIdList: number[] = [];

    // legacy
    if (chainId === "stargaze-0") {
      const tx = (await signingClient.signAndBroadcast(
        owner,
        contractConfigAndStoreCodeMsgList.map((x) => x[1]),
        fee
      )) as unknown as { rawLog: string };

      codeIdList = parseCodeIdListLegacy(tx.rawLog);
    }
    // default
    else {
      const { events } = await signingClient.signAndBroadcast(
        owner,
        contractConfigAndStoreCodeMsgList.map((x) => x[1]),
        fee
      );

      codeIdList = parseCodeIdList(events);
    }

    // update CONFIG with code id's
    for (const i in contractConfigAndStoreCodeMsgList) {
      const [{ WASM }] = contractConfigAndStoreCodeMsgList[i];
      const codeId = codeIdList[i];

      configJson = {
        ...configJson,
        CHAINS: configJson.CHAINS.map((chain) => {
          return {
            ...chain,
            OPTIONS: chain.OPTIONS.map((option) => {
              if (option.CHAIN_ID !== chainId) return option;

              return {
                ...option,
                CONTRACTS: option.CONTRACTS.map((contract) => {
                  if (contract.WASM !== WASM) return contract;

                  return {
                    ...contract,
                    CODE: codeId || 0,
                  };
                }),
              };
            }),
          };
        }),
      };

      const contractName = WASM.replace(".wasm", "").toLowerCase();
      l(`\n"${contractName}" contract code is ${codeId}\n`);
    }

    await writeFile(PATH_TO_CONFIG_JSON, JSON.stringify(configJson), {
      encoding: ENCODING,
    });
  } catch (error) {
    l(error);
  }
}

main();
