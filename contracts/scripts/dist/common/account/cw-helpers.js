import { getLast, l } from "../utils";
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
function getApproveCollectionMsg(collectionAddress, senderAddress, operator) {
  const approveAllMsg = {
    approve_all: {
      operator
    }
  };
  return getSingleTokenExecMsg(collectionAddress, senderAddress, approveAllMsg);
}
function getRevokeCollectionMsg(collectionAddress, senderAddress, operator) {
  const revokeAllMsg = {
    revoke_all: {
      operator
    }
  };
  return getSingleTokenExecMsg(collectionAddress, senderAddress, revokeAllMsg);
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
  async function _msgWrapperWithGasPrice(msgs, gasPrice, gasAdjustment = 1, memo) {
    const tx = await _signAndBroadcast(msgs, gasPrice, gasAdjustment, memo);
    l("\n", tx, "\n");
    return tx;
  }

  // utils

  async function cwRevoke(collectionAddress, senderAddress, operator, gasPrice) {
    return await _msgWrapperWithGasPrice([getRevokeCollectionMsg(collectionAddress, senderAddress, operator)], gasPrice);
  }

  // lending-platform

  async function cwDeposit(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(landingPlatformMsgComposer.deposit(), amount, token)], gasPrice);
  }
  async function cwUnbond(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(landingPlatformMsgComposer.unbond(), amount, token)], gasPrice);
  }
  async function cwWithdraw(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(landingPlatformMsgComposer.withdraw(), amount, token)], gasPrice);
  }
  async function cwApproveAndDepositCollateral(senderAddress, operator, collections, gasPrice) {
    const queryAllOperatorsMsg = {
      all_operators: {
        owner: senderAddress
      }
    };
    let msgList = [];
    for (const {
      collection_address: collectionAddress
    } of collections) {
      const {
        operators
      } = await signingClient.queryContractSmart(collectionAddress, queryAllOperatorsMsg);
      const targetOperator = operators.find(x => x.spender === operator);
      if (!targetOperator) {
        msgList.push(getApproveCollectionMsg(collectionAddress, senderAddress, operator));
      }
    }
    msgList.push(landingPlatformMsgComposer.deposit());
    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }
  async function cwWithdrawCollateral(collections, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.withdrawCollateral({
      collections
    })], gasPrice, 1.05);
  }
  async function cwBorrow(amount, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.borrow({
      amount: `${amount}`
    })], gasPrice);
  }
  async function cwRepay(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(landingPlatformMsgComposer.repay(), amount, token)], gasPrice);
  }
  async function cwPlaceBid(collections, amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(landingPlatformMsgComposer.placeBid({
      collections
    }), amount, token)], gasPrice);
  }
  async function cwRemoveBid(collections, creationDate, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.removeBid({
      collections,
      creationDate
    })], gasPrice);
  }
  async function cwUpdateBid(collections, creationDate, amount, token, gasPrice) {
    // query existing bids
    const queryBidsMsg = {
      query_liquidation_bids_by_liquidator_addresses: {
        addresses: [owner]
      }
    };
    const bids = await signingClient.queryContractSmart(LENDING_PLATFORM_CONTRACT?.ADDRESS || "", queryBidsMsg);
    const currentCollectionsAddresses = collections.map(x => x.collection_address);
    const bidsForCurrentCollections = bids.filter(x => currentCollectionsAddresses.includes(x.address));

    // calculate funds to send
    let amountToSend = 0;
    for (const {
      liquidation_bids
    } of bidsForCurrentCollections) {
      for (const bid of liquidation_bids) {
        const bidAmount = +bid.amount;
        if (amount < bidAmount) {
          amountToSend -= bidAmount - amount;
        } else {
          amountToSend += amount - bidAmount;
        }
      }
    }
    const msgObj = landingPlatformMsgComposer.updateBid({
      collections,
      creationDate,
      amount: `${amount}`
    });
    if (amountToSend > 0) {
      return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(msgObj, amountToSend, token)], gasPrice);
    }
    return await _msgWrapperWithGasPrice([msgObj], gasPrice);
  }
  async function cwClaimCollateral(gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.claimCollateral()], gasPrice);
  }
  async function cwProcess(gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.process()], gasPrice);
  }
  async function cwCreateBglToken(gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.createBglToken()], gasPrice);
  }
  async function cwLock(gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.lock()], gasPrice);
  }
  async function cwUnlock(gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.unlock()], gasPrice);
  }
  async function cwDistributeFunds(addressAndWeightList, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.distributeFunds({
      addressAndWeightList
    })], gasPrice);
  }
  async function cwRemoveCollection(address, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.removeCollection({
      address
    })], gasPrice);
  }
  async function cwCreateProposal(proposal, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.createProposal({
      proposal
    })], gasPrice);
  }
  async function cwRejectProposal(id, gasPrice) {
    return await _msgWrapperWithGasPrice([landingPlatformMsgComposer.rejectProposal({
      id
    })], gasPrice);
  }
  async function cwAcceptProposal(id, amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(landingPlatformMsgComposer.acceptProposal({
      id
    }), amount, token)], gasPrice);
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
    cwDeposit,
    cwUnbond,
    cwWithdraw,
    cwApproveAndDepositCollateral,
    cwWithdrawCollateral,
    cwBorrow,
    cwRepay,
    cwPlaceBid,
    cwRemoveBid,
    cwUpdateBid,
    cwClaimCollateral,
    cwProcess,
    cwCreateBglToken,
    cwLock,
    cwUnlock,
    cwDistributeFunds,
    cwRemoveCollection,
    cwCreateProposal,
    cwRejectProposal,
    cwAcceptProposal,
    cwUpdatePrices,
    cwUpdateConfig,
    cwRevoke
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

  // utils

  async function cwQueryOperators(collectionAddress, ownerAddress) {
    const queryAllOperatorsMsg = {
      all_operators: {
        owner: ownerAddress
      }
    };
    const res = await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryAllOperatorsMsg);
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryApprovals(collectionAddress, tokenId) {
    const queryApprovalsMsg = {
      approvals: {
        token_id: tokenId
      }
    };
    const res = await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryApprovalsMsg);
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBalanceInNft(owner, collectionAddress) {
    const MAX_LIMIT = 100;
    const ITER_LIMIT = 50;
    let tokenList = [];
    let tokenAmountSum = 0;
    let i = 0;
    let lastToken = undefined;
    while ((!i || tokenAmountSum === MAX_LIMIT) && i < ITER_LIMIT) {
      i++;
      try {
        const queryTokensMsg = {
          tokens: {
            owner,
            start_after: lastToken,
            limit: MAX_LIMIT
          }
        };
        const {
          tokens
        } = await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryTokensMsg);
        tokenList = [...tokenList, ...tokens];
        tokenAmountSum = tokens.length;
        lastToken = getLast(tokens);
      } catch (error) {
        l(error);
      }
    }
    const res = {
      tokens: tokenList
    };
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryNftOwner(collectionAddress, tokenId) {
    const queryOwnerOfMsg = {
      owner_of: {
        token_id: tokenId
      }
    };
    const res = await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryOwnerOfMsg);
    l("\n", res, "\n");
    return res;
  }

  // lending-platform

  async function cwQueryPlatformRevenue() {
    const res = await lendingPlatformQueryClient.queryPlatformRevenue();
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBalances() {
    const res = await lendingPlatformQueryClient.queryBalances();
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryDepositors(addresses) {
    const res = await lendingPlatformQueryClient.queryDepositors({
      addresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBorrowers(addresses) {
    const res = await lendingPlatformQueryClient.queryBorrowers({
      addresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidators(addresses) {
    const res = await lendingPlatformQueryClient.queryLiquidators({
      addresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryCollaterals(collectionAddresses) {
    const res = await lendingPlatformQueryClient.queryCollaterals({
      collectionAddresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidationBidsByCollectionAddresses(addresses) {
    const res = await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddresses({
      addresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidationBidsByLiquidatorAddresses(addresses) {
    const res = await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddresses({
      addresses
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryProposals(lastAmount) {
    const res = await lendingPlatformQueryClient.queryProposals({
      lastAmount
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryCollections(addresses) {
    const res = await lendingPlatformQueryClient.queryCollections({
      addresses
    });
    l("\n", res, "\n");
    return res;
  }
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
    cwQueryOperators,
    cwQueryApprovals,
    cwQueryBalanceInNft,
    cwQueryNftOwner,
    cwQueryPlatformRevenue,
    cwQueryBalances,
    cwQueryDepositors,
    cwQueryBorrowers,
    cwQueryLiquidators,
    cwQueryCollaterals,
    cwQueryLiquidationBidsByCollectionAddresses,
    cwQueryLiquidationBidsByLiquidatorAddresses,
    cwQueryProposals,
    cwQueryCollections,
    cwQueryLendingPlatformConfig,
    cwQueryPrices,
    cwQueryOracleConfig
  };
}
export { getCwExecHelpers, getCwQueryHelpers };