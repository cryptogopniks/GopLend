const $ = value => `!${value}!`;
function toJson(obj) {
  return JSON.stringify(obj);
}

// "PREFIX[CHAIN_ID=terra-0]"
// "OPTIONS[CHAIN_ID=terra-0]|GAS_PRICE_AMOUNT"
// "OPTIONS[CHAIN_ID=terra-0]|IBC[COUNTERPARTY_CHAIN_ID=stargaze-0]|CHANNEL_ID"
// "OPTIONS[CHAIN_ID=terra-0]|CONTRACTS[WASM=counter.wasm]|ADDRESS"
function parseTemplate(chainConfig, template) {
  const fieldList = template.split("|");

  // iterate over ChainConfig fields moving to target parameter
  // temp will be updated each iteration
  let targetChain;
  let targetChainOption;
  let targetIbc;
  let targetContract;
  for (let i = 0; i < fieldList.length; i++) {
    const field = fieldList[i];
    const [fieldValue, ...fieldKeyList] = field.split("[");
    if (!fieldKeyList.length) {
      if (i === 1) {
        if (targetChainOption) {
          return targetChainOption[fieldValue];
        }
      }
      if (i === 2) {
        if (targetIbc) {
          return targetIbc[fieldValue];
        }
        if (targetContract) {
          return targetContract[fieldValue];
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
        return targetChain[fieldValue];
      }
    }
    if (i === 1) {
      // iterate over contract or ibc
      if (!targetChainOption) throw new Error("targetChainOption is not found");
      if (fieldValue === "IBC") {
        targetIbc = targetChainOption.IBC.find(x => x[fieldKeyK] === fieldKeyV);
      }
      if (fieldValue === "CONTRACTS") {
        targetContract = targetChainOption.CONTRACTS.find(x => x[fieldKeyK] === fieldKeyV);
      }
    }
  }
  throw new Error("Parameter is not found");
}
function findTemplates(text) {
  const results = [];
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
function replaceTemplates(configJsonObj, config, msgType) {
  for (const {
    OPTIONS
  } of config.CHAINS) {
    for (const {
      CONTRACTS,
      CHAIN_ID
    } of OPTIONS) {
      for (const {
        INIT_MSG,
        UPDATE_MSG,
        MIGRATE_MSG,
        WASM
      } of CONTRACTS) {
        let msg = INIT_MSG;
        if (msgType === "update") {
          msg = UPDATE_MSG;
        }
        if (msgType === "migrate") {
          msg = MIGRATE_MSG;
        }
        for (const template of findTemplates(msg)) {
          const replacement = parseTemplate(configJsonObj, template.replace(/\!/g, ""));
          if (typeof replacement === "number") {
            msg = msg.replace(`"${template}"`, replacement);
          } else {
            msg = msg.replace(template, replacement);
          }
          let msgObj = {
            INIT_MSG: msg
          };
          if (msgType === "update") {
            msgObj = {
              UPDATE_MSG: msg
            };
          }
          if (msgType === "migrate") {
            msgObj = {
              MIGRATE_MSG: msg
            };
          }
          configJsonObj = {
            ...configJsonObj,
            CHAINS: configJsonObj.CHAINS.map(chain => {
              return {
                ...chain,
                OPTIONS: chain.OPTIONS.map(option => {
                  if (option.CHAIN_ID !== CHAIN_ID) return option;
                  return {
                    ...option,
                    CONTRACTS: option.CONTRACTS.map(contract => {
                      if (contract.WASM !== WASM) return contract;
                      return {
                        ...contract,
                        ...msgObj
                      };
                    })
                  };
                })
              };
            })
          };
        }
      }
    }
  }
  return configJsonObj;
}
function getChain(chainConfig, name) {
  const chain = chainConfig.CHAINS.find(x => x.NAME === name);
  if (!chain) throw new Error(`Chain "${name}" is not found!`);
  return chain;
}
function getChainOption(chainConfig, name, type) {
  const {
    OPTIONS
  } = getChain(chainConfig, name);
  const option = OPTIONS.find(x => x.TYPE === type);
  if (!option) {
    throw new Error(`Chain "${name}" "${type}" option is not found!`);
  }
  return option;
}
function getChainOptionById(chainConfig, chainId) {
  let targetOption;
  let name;
  let prefix;
  for (const {
    NAME,
    PREFIX,
    OPTIONS
  } of chainConfig.CHAINS) {
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
  return {
    NAME: name,
    PREFIX: prefix,
    OPTION: targetOption
  };
}
function getContract(chainConfig, name, type, wasm) {
  const {
    CONTRACTS
  } = getChainOption(chainConfig, name, type);
  const contract = CONTRACTS.find(x => x.WASM === wasm);
  if (!contract) throw new Error(`Contract "${contract}" is not found!`);
  return contract;
}
function getContractByWasm(contracts, wasm) {
  const contract = contracts.find(x => x.WASM === wasm);
  if (!contract) throw new Error(`${wasm} is not found!`);
  return contract;
}
export { $, toJson, replaceTemplates, getChain, getChainOption, getContract, getChainOptionById, getContractByWasm };