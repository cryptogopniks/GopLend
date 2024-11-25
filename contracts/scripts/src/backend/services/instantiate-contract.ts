import { l } from "../../common/utils";
import { calculateFee, coins, Event } from "@cosmjs/stargate";
import { toUtf8 } from "@cosmjs/encoding";
import { CHAIN_CONFIG } from "../../common/config";
import { readFile, writeFile } from "fs/promises";
import { getCwClient } from "../../common/account/clients";
import { getSigner } from "../account/signer";
import { ContractInfo, ChainConfig } from "../../common/interfaces";
import { MsgInstantiateContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { LEGACY_CHAIN_ID_LIST } from "./constants";
import {
  SigningCosmWasmClient,
  MsgInstantiateContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";
import {
  getChainOptionById,
  replaceTemplates,
} from "../../common/config/config-utils";
import {
  parseStoreArgs,
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
} from "./utils";

function parseAddressListLegacy(rawLog: string): string[] {
  const regex = /"_contract_address","value":"(\w+)"/g;

  const addresses = (rawLog.match(regex) || []).map((x) =>
    x.split(":")[1].replace(/"/g, "")
  );

  return [...new Set(addresses).values()];
}

function parseAddressList(events: readonly Event[]): string[] {
  return events
    .filter((x) => x.type === "instantiate")
    .map(
      (x) =>
        x.attributes.find((y) => y.key === "_contract_address")?.value || ""
    );
}

async function main() {
  try {
    const { chainId, labelList } = parseStoreArgs();
    const configJsonStr: string = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    let configJson: ChainConfig = JSON.parse(configJsonStr);
    configJson = replaceTemplates(
      chainId,
      configJson,
      CHAIN_CONFIG,
      "instantiate"
    );

    const {
      PREFIX,
      OPTION: {
        DENOM,
        RPC_LIST: [RPC],
        GAS_PRICE_AMOUNT,
        CONTRACTS,
        TYPE,
      },
    } = getChainOptionById(configJson, chainId);

    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);
    const cwClient = await getCwClient(RPC, owner, signer);
    if (!cwClient) throw new Error("cwClient is not found!");

    const signingClient = cwClient.client as SigningCosmWasmClient;

    let contractConfigAndInitMsgList: [
      ContractInfo,
      MsgInstantiateContractEncodeObject
    ][] = [];
    let addressList: string[] = [];

    for (const CONTRACT of CONTRACTS) {
      if (!labelList.includes(CONTRACT.LABEL)) continue;

      const { LABEL, INIT_MSG, CODE } = CONTRACT;

      const instantiateContractMsg: MsgInstantiateContractEncodeObject = {
        typeUrl: "/cosmwasm.wasm.v1.MsgInstantiateContract",
        value: MsgInstantiateContract.fromPartial({
          sender: owner,
          codeId: BigInt(CODE),
          label: LABEL,
          msg: toUtf8(INIT_MSG),
          funds: [],
          admin: owner,
        }),
      };

      contractConfigAndInitMsgList.push([CONTRACT, instantiateContractMsg]);
    }

    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const gasSimulated = await signingClient.simulate(
      owner,
      contractConfigAndInitMsgList.map((x) => x[1]),
      ""
    );
    const gasWantedSim = Math.ceil(1.2 * gasSimulated);

    // legacy
    if (LEGACY_CHAIN_ID_LIST.includes(chainId)) {
      const tx = (await signingClient.signAndBroadcast(
        owner,
        contractConfigAndInitMsgList.map((x) => x[1]),
        calculateFee(gasWantedSim, gasPrice)
      )) as unknown as { rawLog: string };

      addressList = parseAddressListLegacy(tx.rawLog);
    }
    // default
    else {
      const { events } = await signingClient.signAndBroadcast(
        owner,
        contractConfigAndInitMsgList.map((x) => x[1]),
        calculateFee(gasWantedSim, gasPrice)
      );

      addressList = parseAddressList(events);
    }

    // update CONFIG with contract addresses
    for (const i in contractConfigAndInitMsgList) {
      const [{ LABEL }] = contractConfigAndInitMsgList[i];
      const contractAddress = addressList[i];

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
                  if (contract.LABEL !== LABEL) return contract;

                  return {
                    ...contract,
                    ADDRESS: contractAddress || "",
                  };
                }),
              };
            }),
          };
        }),
      };

      const contractName = LABEL.toLowerCase();
      l(`"${contractName}" contract address is ${contractAddress}\n`);
    }

    await writeFile(PATH_TO_CONFIG_JSON, JSON.stringify(configJson, null, 2), {
      encoding: ENCODING,
    });
  } catch (error) {
    l(error);
  }
}

main();
