import { Label } from ".";
import {
  ChainConfig,
  ChainType,
  Wasm,
  ChainOption,
  ContractInfo,
  IbcConfig,
  ChainItem,
} from "../interfaces";

const $ = (value: string): any => `!${value}!` as any;

function toJson<T>(obj: T): string {
  return JSON.stringify(obj);
}

// "PREFIX[CHAIN_ID=terra-0]"
// "OPTIONS[CHAIN_ID=terra-0]|GAS_PRICE_AMOUNT"
// "OPTIONS[CHAIN_ID=terra-0]|IBC[COUNTERPARTY_CHAIN_ID=stargaze-0]|CHANNEL_ID"
// "OPTIONS[CHAIN_ID=terra-0]|CONTRACTS[LABEL=counter]|ADDRESS"
function parseTemplate(chainConfig: ChainConfig, template: string) {
  const fieldList = template.split("|");

  // iterate over ChainConfig fields moving to target parameter
  // temp will be updated each iteration
  let targetChain: ChainItem | undefined;
  let targetChainOption: ChainOption | undefined;
  let targetIbc: IbcConfig | undefined;
  let targetContract: ContractInfo | undefined;

  for (let i = 0; i < fieldList.length; i++) {
    const field = fieldList[i];
    const [fieldValue, ...fieldKeyList] = field.split("[");

    if (!fieldKeyList.length) {
      if (i === 1) {
        if (targetChainOption) {
          return targetChainOption[fieldValue as keyof ChainOption];
        }
      }

      if (i === 2) {
        if (targetIbc) {
          return targetIbc[fieldValue as keyof IbcConfig];
        }

        if (targetContract) {
          return targetContract[fieldValue as keyof ContractInfo];
        }
      }
    }

    const [fieldKeyK, fieldKeyV] = fieldKeyList[0].replace("]", "").split("=");

    if (!i) {
      // iterate over options
      for (const chain of chainConfig.CHAINS) {
        for (const option of chain.OPTIONS) {
          if (option.CHAIN_ID === fieldKeyV) {
            targetChain = chain;
            targetChainOption = option;
            break;
          }
        }
      }

      if (targetChain && fieldList.length === 1) {
        return targetChain[fieldValue as keyof ChainItem];
      }
    }

    if (i === 1) {
      // iterate over contract or ibc
      if (!targetChainOption) throw new Error("targetChainOption is not found");

      if (fieldValue === "IBC") {
        targetIbc = targetChainOption.IBC.find(
          (x) => x[fieldKeyK as keyof IbcConfig] === fieldKeyV
        );
      }

      if (fieldValue === "CONTRACTS") {
        targetContract = targetChainOption.CONTRACTS.find(
          (x) => x[fieldKeyK as keyof ContractInfo] === fieldKeyV
        );
      }
    }
  }

  throw new Error("Parameter is not found");
}

function findTemplates(text: string): string[] {
  const results: string[] = [];
  let lastIndex = 0;
  let index;

  // Iterate through string to find "!"
  while ((index = text.indexOf("!", lastIndex)) !== -1) {
    const start = index;

    // Find next "!"
    const end = text.indexOf("!", index + 1);
    if (end !== -1) {
      // Extract text between "!" and add to results
      results.push(text.substring(start, end + 1));
      lastIndex = end + 1;
    }
  }

  return results;
}

type MsgType = "instantiate" | "update" | "migrate";

function replaceTemplates(
  chainId: string,
  configJsonObj: ChainConfig,
  config: ChainConfig,
  msgType: MsgType
): ChainConfig {
  for (const { OPTIONS } of config.CHAINS) {
    for (const { CONTRACTS, CHAIN_ID } of OPTIONS) {
      for (const { INIT_MSG, UPDATE_MSG, MIGRATE_MSG, LABEL } of CONTRACTS) {
        let msg = INIT_MSG;

        if (msgType === "update") {
          msg = UPDATE_MSG;
        }

        if (msgType === "migrate") {
          msg = MIGRATE_MSG;
        }

        for (const template of findTemplates(msg)) {
          // ignore templates for other chains
          if (!template.includes(chainId)) continue;

          const replacement = parseTemplate(
            configJsonObj,
            template.replace(/\!/g, "")
          );

          if (typeof replacement === "number") {
            msg = msg.replace(`"${template}"`, replacement as any);
          } else {
            msg = msg.replace(template, replacement as any);
          }

          let msgObj:
            | { INIT_MSG: string }
            | { UPDATE_MSG: string }
            | { MIGRATE_MSG: string } = { INIT_MSG: msg };

          if (msgType === "update") {
            msgObj = { UPDATE_MSG: msg };
          }

          if (msgType === "migrate") {
            msgObj = { MIGRATE_MSG: msg };
          }

          configJsonObj = {
            ...configJsonObj,
            CHAINS: configJsonObj.CHAINS.map((chain) => {
              return {
                ...chain,
                OPTIONS: chain.OPTIONS.map((option) => {
                  if (option.CHAIN_ID !== CHAIN_ID) return option;

                  return {
                    ...option,
                    CONTRACTS: option.CONTRACTS.map((contract) => {
                      if (contract.LABEL !== LABEL) return contract;

                      return { ...contract, ...msgObj };
                    }),
                  };
                }),
              };
            }),
          };
        }
      }
    }
  }

  return configJsonObj;
}

function getChain(chainConfig: ChainConfig, name: string) {
  const chain = chainConfig.CHAINS.find((x) => x.NAME === name);
  if (!chain) throw new Error(`Chain "${name}" is not found!`);

  return chain;
}

function getChainOption(
  chainConfig: ChainConfig,
  name: string,
  type: ChainType
) {
  const { OPTIONS } = getChain(chainConfig, name);
  const option = OPTIONS.find((x) => x.TYPE === type);
  if (!option) {
    throw new Error(`Chain "${name}" "${type}" option is not found!`);
  }

  return option;
}

function getChainOptionById(chainConfig: ChainConfig, chainId: string) {
  let targetOption: ChainOption | undefined;
  let name: string | undefined;
  let prefix: string | undefined;

  for (const { NAME, PREFIX, OPTIONS } of chainConfig.CHAINS) {
    for (const option of OPTIONS) {
      if (option.CHAIN_ID === chainId) {
        targetOption = option;
        name = NAME;
        prefix = PREFIX;
        break;
      }
    }
  }

  if (!(targetOption && name && prefix)) {
    throw new Error(`Chain "${chainId}" option is not found!`);
  }

  return { NAME: name, PREFIX: prefix, OPTION: targetOption };
}

function getContract(
  chainConfig: ChainConfig,
  name: string,
  type: ChainType,
  label: Label
) {
  const { CONTRACTS } = getChainOption(chainConfig, name, type);
  const contract = CONTRACTS.find((x) => x.LABEL === label);
  if (!contract) throw new Error(`Contract "${contract}" is not found!`);

  return contract;
}

function getContractByWasm(contracts: ContractInfo[], wasm: Wasm) {
  const contract = contracts.find((x) => x.WASM === wasm);
  if (!contract) throw new Error(`${wasm} is not found!`);

  return contract;
}

function getContractByLabel(contracts: ContractInfo[], label: Label) {
  const contract = contracts.find((x) => x.LABEL === label);
  if (!contract) throw new Error(`${label} is not found!`);

  return contract;
}

export {
  $,
  toJson,
  replaceTemplates,
  getChain,
  getChainOption,
  getContract,
  getChainOptionById,
  getContractByWasm,
  getContractByLabel,
};
