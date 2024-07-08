import { getSigner } from "../account/signer";
import { l } from "../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig, ContractInfo } from "../../common/interfaces";
import { CHAIN_CONFIG } from "../../common/config";
import { calculateFee } from "@cosmjs/stargate";
import { toUtf8 } from "@cosmjs/encoding";
import { getCwClient } from "../../common/account/clients";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import {
  getChainOptionById,
  replaceTemplates,
} from "../../common/config/config-utils";
import {
  SigningCosmWasmClient,
  MsgExecuteContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";
import {
  parseStoreArgs,
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
} from "./utils";

async function main() {
  try {
    const { chainId, labelList } = parseStoreArgs();
    const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    let configJson: ChainConfig = JSON.parse(configJsonStr);
    configJson = replaceTemplates(chainId, configJson, CHAIN_CONFIG, "update");

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

    let contractConfigAndUpdateMsgList: [
      ContractInfo,
      MsgExecuteContractEncodeObject
    ][] = [];

    for (const CONTRACT of CONTRACTS) {
      if (!labelList.includes(CONTRACT.LABEL)) continue;

      const { UPDATE_MSG, ADDRESS } = CONTRACT;
      const msg: MsgExecuteContractEncodeObject = {
        typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
        value: MsgExecuteContract.fromPartial({
          sender: owner,
          contract: ADDRESS,
          msg: toUtf8(UPDATE_MSG),
          funds: [],
        }),
      };

      contractConfigAndUpdateMsgList.push([CONTRACT, msg]);
    }

    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const gasSimulated = await signingClient.simulate(
      owner,
      contractConfigAndUpdateMsgList.map((x) => x[1]),
      ""
    );
    const gasWantedSim = Math.ceil(1.3 * gasSimulated);

    const tx = await signingClient.signAndBroadcast(
      owner,
      contractConfigAndUpdateMsgList.map((x) => x[1]),
      calculateFee(gasWantedSim, gasPrice)
    );
    l(tx);

    await writeFile(PATH_TO_CONFIG_JSON, JSON.stringify(configJson, null, 2), {
      encoding: ENCODING,
    });
  } catch (error) {
    l(error);
  }
}

main();
