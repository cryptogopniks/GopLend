import { l } from "../utils";
import { toBase64, fromUtf8 } from "@cosmjs/encoding";
import { LendingPlatformMsgComposer } from "../codegen/LendingPlatform.message-composer";
import { OracleMsgComposer } from "../codegen/Oracle.message-composer";
import { LendingPlatformQueryClient } from "../codegen/LendingPlatform.client";
import { OracleQueryClient } from "../codegen/Oracle.client";
import CONFIG_JSON from "../config/config.json";
import { getChainOptionById, getContractByWasm } from "../config/config-utils";
import { getCwClient, signAndBroadcastWrapper, getExecuteContractMsg } from "./clients";
import { coin } from "@cosmjs/proto-signing";
function addSingleTokenToComposerObj(obj, amount, token) {
  const {
    value: {
      contract,
      sender,
      msg
    }
  } = obj;
  if (!(contract && sender && msg)) {
    throw new Error(`${msg} parameters error!`);
  }
  return getSingleTokenExecMsg(contract, sender, JSON.parse(fromUtf8(msg)), amount, token);
}
function getSingleTokenExecMsg(contractAddress, senderAddress, msg, amount, token) {
  // get msg without funds
  if (!(token && amount)) {
    return getExecuteContractMsg(contractAddress, senderAddress, msg, []);
  }

  // get msg with native token
  if ("native" in token) {
    return getExecuteContractMsg(contractAddress, senderAddress, msg, [coin(amount, token.native.denom)]);
  }

  // get msg with CW20 token
  const cw20SendMsg = {
    send: {
      contract: contractAddress,
      amount: `${amount}`,
      msg: toBase64(msg)
    }
  };
  return getExecuteContractMsg(token.cw20.address, senderAddress, cw20SendMsg, []);
}
async function getCwExecHelpers(chainId, rpc, owner, signer) {
  const CHAIN_CONFIG = CONFIG_JSON;
  const {
    OPTION: {
      CONTRACTS
    }
  } = getChainOptionById(CHAIN_CONFIG, chainId);
  let LENDING_PLATFORM_CONTRACT;
  let ORACLE_CONTRACT;
  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(CONTRACTS, "lending_platform.wasm");
  } catch (error) {
    l(error);
  }
  try {
    ORACLE_CONTRACT = getContractByWasm(CONTRACTS, "oracle.wasm");
  } catch (error) {
    l(error);
  }
  const cwClient = await getCwClient(rpc, owner, signer);
  if (!cwClient) throw new Error("cwClient is not found!");
  const signingClient = cwClient.client;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);
  const landingPlatformMsgComposer = new LendingPlatformMsgComposer(owner, LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : "");
  const oracleMsgComposer = new OracleMsgComposer(owner, ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : "");
  async function _msgWrapperWithGasPrice(msgs, gasPrice) {
    const tx = await _signAndBroadcast(msgs, gasPrice);
    l("\n", tx, "\n");
    return tx;
  }

  // oracle

  async function cwUpdatePrices(data, gasPrice) {
    return await _msgWrapperWithGasPrice([oracleMsgComposer.updatePrices({
      data
    })], gasPrice);
  }
  async function cwUpdateConfig(updateConfigStruct, gasPrice) {
    let msgList = [];
    const {
      bglCurrency,
      bidMinRate,
      borrowApr,
      borrowFeeRate,
      cw20BaseCodeId,
      discountMaxRate,
      discountMinRate,
      liquidationFeeRate,
      mainCurrency,
      oracle,
      owner,
      schedulerForLendingPlatform,
      unbondingPeriod,
      schedulerForOracle
    } = updateConfigStruct;
    if (schedulerForOracle) {
      msgList.push(oracleMsgComposer.updateConfig({
        scheduler: schedulerForOracle
      }));
    }
    if (bglCurrency || bidMinRate || borrowApr || borrowFeeRate || cw20BaseCodeId || discountMaxRate || discountMinRate || liquidationFeeRate || mainCurrency || oracle || owner || schedulerForLendingPlatform || unbondingPeriod) {
      msgList.push(landingPlatformMsgComposer.updateConfig({
        bglCurrency,
        bidMinRate,
        borrowApr,
        borrowFeeRate,
        cw20BaseCodeId,
        discountMaxRate,
        discountMinRate,
        liquidationFeeRate,
        mainCurrency,
        oracle,
        owner,
        scheduler: schedulerForLendingPlatform,
        unbondingPeriod
      }));
    }
    if (!msgList.length) {
      throw new Error("cwUpdateConfig arguments are not provided!");
    }
    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }
  return {
    cwUpdatePrices,
    cwUpdateConfig
  };
}
async function getCwQueryHelpers(chainId, rpc) {
  const CHAIN_CONFIG = CONFIG_JSON;
  const {
    OPTION: {
      CONTRACTS
    }
  } = getChainOptionById(CHAIN_CONFIG, chainId);
  let LENDING_PLATFORM_CONTRACT;
  let ORACLE_CONTRACT;
  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(CONTRACTS, "lending_platform.wasm");
  } catch (error) {
    l(error);
  }
  try {
    ORACLE_CONTRACT = getContractByWasm(CONTRACTS, "oracle.wasm");
  } catch (error) {
    l(error);
  }
  const cwClient = await getCwClient(rpc);
  if (!cwClient) throw new Error("cwClient is not found!");
  const cosmwasmQueryClient = cwClient.client;
  const lendingPlatformQueryClient = new LendingPlatformQueryClient(cosmwasmQueryClient, LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : "");
  const oracleQueryClient = new OracleQueryClient(cosmwasmQueryClient, ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : "");

  // lending-platform

  async function cwQueryLendingPlatformConfig() {
    const res = await lendingPlatformQueryClient.queryConfig();
    l("\n", res, "\n");
    return res;
  }

  // oracle

  async function cwQueryPrices(collectionAddresses) {
    const res = await oracleQueryClient.queryPrices({
      collectionAddresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryOracleConfig() {
    const res = await oracleQueryClient.queryConfig();
    l("\n", res, "\n");
    return res;
  }
  return {
    cwQueryLendingPlatformConfig,
    cwQueryPrices,
    cwQueryOracleConfig
  };
}
export { getCwExecHelpers, getCwQueryHelpers };