import { getLast, l } from "../utils";
import { toBase64, fromUtf8 } from "@cosmjs/encoding";
import { AdapterSchedulerKujiraMsgComposer } from "../codegen/AdapterSchedulerKujira.message-composer";
import { LendingPlatformMsgComposer } from "../codegen/LendingPlatform.message-composer";
import { MinterMsgComposer } from "../codegen/Minter.message-composer";
import { OracleMsgComposer } from "../codegen/Oracle.message-composer";
import { AdapterSchedulerKujiraQueryClient } from "../codegen/AdapterSchedulerKujira.client";
import { LendingPlatformQueryClient } from "../codegen/LendingPlatform.client";
import { MinterQueryClient } from "../codegen/Minter.client";
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
  let ADAPTER_SCHEDULER_KUJIRA_CONTRACT;
  // let ADAPTER_SCHEDULER_STARGAZE_CONTRACT: ContractInfo | undefined;
  let LENDING_PLATFORM_CONTRACT;
  let MINTER_CONTRACT;
  let ORACLE_CONTRACT;
  try {
    ADAPTER_SCHEDULER_KUJIRA_CONTRACT = getContractByWasm(CONTRACTS, "adapter_scheduler_kujira.wasm");
  } catch (error) {
    l(error);
  }
  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(CONTRACTS, "lending_platform.wasm");
  } catch (error) {
    l(error);
  }
  try {
    MINTER_CONTRACT = getContractByWasm(CONTRACTS, "minter.wasm");
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
  const adapterSchedulerKujiraMsgComposer = new AdapterSchedulerKujiraMsgComposer(owner, ADAPTER_SCHEDULER_KUJIRA_CONTRACT ? ADAPTER_SCHEDULER_KUJIRA_CONTRACT.ADDRESS : "");
  const lendingPlatformMsgComposer = new LendingPlatformMsgComposer(owner, LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : "");
  const minterMsgComposer = new MinterMsgComposer(owner, MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : "");
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

  // adapter-scheduler-kujira

  async function cwAdapterSchedulerKujiraUpdateConfig({
    clockSource,
    lendingPlatform,
    minClockPeriod,
    offchainClock
  }, gasPrice) {
    return await _msgWrapperWithGasPrice([adapterSchedulerKujiraMsgComposer.updateConfig({
      lendingPlatform,
      offchainClock,
      clockSource,
      minClockPeriod
    })], gasPrice);
  }
  async function cwPush(targets, gasPrice) {
    return await _msgWrapperWithGasPrice([adapterSchedulerKujiraMsgComposer.push({
      targets
    })], gasPrice);
  }

  // lending-platform

  async function cwDeposit(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.deposit(), amount, token)], gasPrice);
  }
  async function cwUnbond(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.unbond(), amount, token)], gasPrice);
  }
  async function cwWithdraw(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.withdraw(), amount, token)], gasPrice);
  }
  async function cwWithdrawCollateral(collections, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.withdrawCollateral({
      collections
    })], gasPrice, 1.05);
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
    msgList.push(lendingPlatformMsgComposer.deposit());
    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }
  async function cwBorrow(amount, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.borrow({
      amount: `${amount}`
    })], gasPrice);
  }
  async function cwRepay(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.repay(), amount, token)], gasPrice);
  }
  async function cwPlaceBid(collections, discount, amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.placeBid({
      collections,
      discount: discount.toString()
    }), amount, token)], gasPrice);
  }
  async function cwRemoveBid(collections, creationDate, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.removeBid({
      collections,
      creationDate
    })], gasPrice);
  }
  async function cwUpdateBid(collections, creationDate, amount, discount, token, gasPrice) {
    // query existing bids
    const queryBidsMsg = {
      query_liquidation_bids_by_liquidator_address: {
        address: owner
      }
    };
    const bids = await signingClient.queryContractSmart(LENDING_PLATFORM_CONTRACT?.ADDRESS || "", queryBidsMsg);
    const currentCollectionsAddresses = collections.map(x => x.collection_address);
    const bidsForCurrentCollections = bids.filter(x => currentCollectionsAddresses.includes(x.collection_address));

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
    const msgObj = lendingPlatformMsgComposer.updateBid({
      collections,
      creationDate,
      amount: amount.toString(),
      discount: discount.toString()
    });
    if (amountToSend > 0) {
      return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(msgObj, amountToSend, token)], gasPrice);
    }
    return await _msgWrapperWithGasPrice([msgObj], gasPrice);
  }
  async function cwLiquidate(targets, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.liquidate({
      targets
    })], gasPrice);
  }
  async function cwLendingPlatformUpdateAddressConfig({
    marketplace,
    minter,
    oracle,
    owner,
    scheduler
  }, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.updateAddressConfig({
      marketplace,
      minter,
      oracle,
      owner,
      scheduler
    })], gasPrice);
  }
  async function cwLendingPlatformUpdateRateConfig({
    bidMinRate,
    borrowApr,
    borrowFeeRate,
    discountMaxRate,
    discountMinRate,
    liquidationFeeRate
  }, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.updateRateConfig({
      bidMinRate: bidMinRate?.toString(),
      borrowApr: borrowApr?.toString(),
      borrowFeeRate: borrowFeeRate?.toString(),
      discountMaxRate: discountMaxRate?.toString(),
      discountMinRate: discountMinRate?.toString(),
      liquidationFeeRate: liquidationFeeRate?.toString()
    })], gasPrice);
  }
  async function cwLendingPlatformUpdateCommonConfig({
    bglCurrency,
    collateralMinValue,
    isMarketplaceInterfaceEnabled,
    mainCurrency,
    minAvailableLiquidityMultiplier,
    priceUpdatePeriod,
    unbondingPeriod
  }, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.updateCommonConfig({
      bglCurrency,
      collateralMinValue: collateralMinValue?.toString(),
      isMarketplaceInterfaceEnabled,
      mainCurrency,
      minAvailableLiquidityMultiplier: minAvailableLiquidityMultiplier?.toString(),
      priceUpdatePeriod,
      unbondingPeriod
    })], gasPrice);
  }
  async function cwDepositReserveLiquidity(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.depositReserveLiquidity(), amount, token)], gasPrice);
  }
  async function cwReinforceBglToken(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.reinforceBglToken(), amount, token)], gasPrice);
  }
  async function cwLock(gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.lock()], gasPrice);
  }
  async function cwUnlock(gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.unlock()], gasPrice);
  }
  async function cwDistributeFunds(addressAndWeightList, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.distributeFunds({
      addressAndWeightList: addressAndWeightList.map(([address, weight]) => [address, weight.toString()])
    })], gasPrice);
  }
  async function cwRemoveCollection(address, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.removeCollection({
      address
    })], gasPrice);
  }
  async function cwCreateProposal(proposal, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.createProposal({
      proposal
    })], gasPrice);
  }
  async function cwRejectProposal(id, gasPrice) {
    return await _msgWrapperWithGasPrice([lendingPlatformMsgComposer.rejectProposal({
      id
    })], gasPrice);
  }
  async function cwAcceptProposal(id, amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(lendingPlatformMsgComposer.acceptProposal({
      id
    }), amount, token)], gasPrice);
  }

  // minter

  async function cwCreateNative(tokenOwner, subdenom, decimals, paymentAmount, paymentDenom, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(minterMsgComposer.createNative({
      tokenOwner,
      subdenom,
      decimals
    }), paymentAmount, {
      native: {
        denom: paymentDenom
      }
    })], gasPrice);
  }
  async function cwCreateCw20(tokenOwner, name, symbol, marketing, decimals = 6, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.createCw20({
      tokenOwner,
      name,
      symbol,
      decimals,
      marketing
    })], gasPrice);
  }
  async function cwMint(amount, recipient, token, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.mint({
      token,
      amount: amount.toString(),
      recipient
    })], gasPrice);
  }
  async function cwBurn(amount, token, gasPrice) {
    return await _msgWrapperWithGasPrice([addSingleTokenToComposerObj(minterMsgComposer.burn(), amount, token)], gasPrice);
  }
  async function cwSetMetadataNative(token, metadata, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.setMetadataNative({
      token,
      metadata
    })], gasPrice);
  }
  async function cwUpdateMarketingCw20(token, description, marketing, project, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.updateMarketingCw20({
      token,
      project,
      description,
      marketing
    })], gasPrice);
  }
  async function cwUploadLogoCw20(token, logo, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.uploadLogoCw20({
      token,
      logo
    })], gasPrice);
  }
  async function cwChangeAdminNative(token, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.changeAdminNative({
      token
    })], gasPrice);
  }
  async function cwUpdateMinterCw20(token, newMinter, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.updateMinterCw20({
      token,
      newMinter
    })], gasPrice);
  }
  async function cwRegisterCurrency(currency, creator, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.registerCurrency({
      currency,
      creator
    })], gasPrice);
  }
  async function cwMinterUpdateConfig({
    cw20BaseCodeId,
    owner
  }, gasPrice) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.updateConfig({
      owner,
      cw20BaseCodeId
    })], gasPrice);
  }

  // oracle

  async function cwOracleUpdateConfig(scheduler, gasPrice) {
    return await _msgWrapperWithGasPrice([oracleMsgComposer.updateConfig({
      scheduler
    })], gasPrice);
  }
  async function cwUpdatePrices(data, gasPrice) {
    return await _msgWrapperWithGasPrice([oracleMsgComposer.updatePrices({
      data
    })], gasPrice);
  }
  return {
    utils: {
      cwRevoke
    },
    adapter: {
      scheduler: {
        kujira: {
          cwUpdateConfig: cwAdapterSchedulerKujiraUpdateConfig,
          cwPush
        }
      }
    },
    lending: {
      cwDeposit,
      cwUnbond,
      cwWithdraw,
      cwWithdrawCollateral,
      cwApproveAndDepositCollateral,
      cwBorrow,
      cwRepay,
      cwPlaceBid,
      cwRemoveBid,
      cwUpdateBid,
      cwLiquidate,
      cwUpdateAddressConfig: cwLendingPlatformUpdateAddressConfig,
      cwUpdateRateConfig: cwLendingPlatformUpdateRateConfig,
      cwUpdateCommonConfig: cwLendingPlatformUpdateCommonConfig,
      cwDepositReserveLiquidity,
      cwReinforceBglToken,
      cwLock,
      cwUnlock,
      cwDistributeFunds,
      cwRemoveCollection,
      cwCreateProposal,
      cwRejectProposal,
      cwAcceptProposal
    },
    minter: {
      cwCreateNative,
      cwCreateCw20,
      cwMint,
      cwBurn,
      cwSetMetadataNative,
      cwUpdateMarketingCw20,
      cwUploadLogoCw20,
      cwChangeAdminNative,
      cwUpdateMinterCw20,
      cwRegisterCurrency,
      cwUpdateConfig: cwMinterUpdateConfig
    },
    oracle: {
      cwUpdateConfig: cwOracleUpdateConfig,
      cwUpdatePrices
    }
  };
}
async function getCwQueryHelpers(chainId, rpc) {
  const CHAIN_CONFIG = CONFIG_JSON;
  const {
    OPTION: {
      CONTRACTS
    }
  } = getChainOptionById(CHAIN_CONFIG, chainId);
  let ADAPTER_SCHEDULER_KUJIRA_CONTRACT;
  let LENDING_PLATFORM_CONTRACT;
  let MINTER_CONTRACT;
  let ORACLE_CONTRACT;
  try {
    ADAPTER_SCHEDULER_KUJIRA_CONTRACT = getContractByWasm(CONTRACTS, "adapter_scheduler_kujira.wasm");
  } catch (error) {
    l(error);
  }
  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(CONTRACTS, "lending_platform.wasm");
  } catch (error) {
    l(error);
  }
  try {
    MINTER_CONTRACT = getContractByWasm(CONTRACTS, "minter.wasm");
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
  const adapterSchedulerKujiraQueryClient = new AdapterSchedulerKujiraQueryClient(cosmwasmQueryClient, ADAPTER_SCHEDULER_KUJIRA_CONTRACT ? ADAPTER_SCHEDULER_KUJIRA_CONTRACT.ADDRESS : "");
  const lendingPlatformQueryClient = new LendingPlatformQueryClient(cosmwasmQueryClient, LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : "");
  const minterQueryClient = new MinterQueryClient(cosmwasmQueryClient, MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : "");
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

  // adapter-scheduler-kujira

  async function cwAdapterSchedulerKujiraQueryConfig() {
    const res = await adapterSchedulerKujiraQueryClient.queryConfig();
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLog(amount) {
    const res = await adapterSchedulerKujiraQueryClient.queryLog({
      amount
    });
    l("\n", res, "\n");
    return res;
  }

  // lending-platform

  async function cwLendingPlatformQueryAddressConfig() {
    const res = await lendingPlatformQueryClient.queryAddressConfig();
    l("\n", res, "\n");
    return res;
  }
  async function cwLendingPlatformQueryRateConfig() {
    const res = await lendingPlatformQueryClient.queryRateConfig();
    l("\n", res, "\n");
    return res;
  }
  async function cwLendingPlatformQueryCommonConfig() {
    const res = await lendingPlatformQueryClient.queryCommonConfig();
    l("\n", res, "\n");
    return res;
  }
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
  async function cwQueryUnbonderList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryUnbonderList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryUnbonder(address) {
    const res = await lendingPlatformQueryClient.queryUnbonder({
      address
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBorrowerList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryBorrowerList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBorrower(address) {
    const res = await lendingPlatformQueryClient.queryBorrower({
      address
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidatorList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryLiquidatorList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidator(address) {
    const res = await lendingPlatformQueryClient.queryLiquidator({
      address
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryCollateralList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryCollateralList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryCollateral(collectionAddress) {
    const res = await lendingPlatformQueryClient.queryCollateral({
      collectionAddress
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidationBidsByCollectionAddressList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddressList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidationBidsByCollectionAddress(address) {
    const res = await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddress({
      address
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidationBidsByLiquidatorAddressList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddressList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLiquidationBidsByLiquidatorAddress(address) {
    const res = await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddress({
      address
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryProposals(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryProposals({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryCollectionList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryCollectionList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryCollection(address) {
    const res = await lendingPlatformQueryClient.queryCollection({
      address
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBglCurrencyToMainCurrencyPrice() {
    const res = await lendingPlatformQueryClient.queryBglCurrencyToMainCurrencyPrice();
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryConditionalDepositApr(amountToDeposit) {
    const res = await lendingPlatformQueryClient.queryConditionalDepositApr({
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryLtvList(limit = 10_000, startAfter) {
    const res = await lendingPlatformQueryClient.queryLtvList({
      startAfter,
      limit
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryConditionalLtv(borrower, amountToDeposit = 0, amountToBorrow = 0) {
    const res = await lendingPlatformQueryClient.queryConditionalLtv({
      borrower,
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
      amountToBorrow: amountToBorrow ? amountToBorrow.toString() : undefined
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryTotalAvailableToBorrowLiquidity() {
    const res = await lendingPlatformQueryClient.queryTotalAvailableToBorrowLiquidity();
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryAvailableToBorrow(borrower, targetLtv) {
    const res = await lendingPlatformQueryClient.queryAvailableToBorrow({
      borrower,
      targetLtv: targetLtv ? targetLtv.toString() : undefined
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryAmounts() {
    const res = await lendingPlatformQueryClient.queryAmounts();
    l("\n", res, "\n");
    return res;
  }

  // async function cwQueryTargets(borrowers: string[]) {
  //   const res = await lendingPlatformQueryClient.queryTargets({ borrowers });
  //   l("\n", res, "\n");
  //   return res;
  // }

  // minter

  async function cwQueryCurrenciesByCreator(creator) {
    const res = await minterQueryClient.queryCurrenciesByCreator({
      creator
    });
    l("\n", res, "\n");
    return res;
  }
  async function cwMinterQueryConfig() {
    const res = await minterQueryClient.queryConfig();
    l("\n", res, "\n");
    return res;
  }

  // oracle

  async function cwOracleQueryConfig() {
    const res = await oracleQueryClient.queryConfig();
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryPrices(limit = 10_000, startAfter, collectionAddresses = []) {
    const msg_1 = {
      startAfter,
      limit
    };
    const msg_2 = {
      collectionAddresses
    };
    const res = await oracleQueryClient.queryPrices(collectionAddresses.length ? msg_2 : msg_1);
    l("\n", res, "\n");
    return res;
  }
  async function cwQueryBlockTime() {
    const res = await oracleQueryClient.queryBlockTime();
    l("\n", res, "\n");
    return res;
  }
  return {
    utils: {
      cwQueryOperators,
      cwQueryApprovals,
      cwQueryBalanceInNft,
      cwQueryNftOwner
    },
    adapter: {
      scheduler: {
        kujira: {
          cwQueryConfig: cwAdapterSchedulerKujiraQueryConfig,
          cwQueryLog
        }
      }
    },
    lending: {
      cwQueryAddressConfig: cwLendingPlatformQueryAddressConfig,
      cwQueryRateConfig: cwLendingPlatformQueryRateConfig,
      cwQueryCommonConfig: cwLendingPlatformQueryCommonConfig,
      cwQueryPlatformRevenue,
      cwQueryBalances,
      cwQueryUnbonderList,
      cwQueryUnbonder,
      cwQueryBorrowerList,
      cwQueryBorrower,
      cwQueryLiquidatorList,
      cwQueryLiquidator,
      cwQueryCollateralList,
      cwQueryCollateral,
      cwQueryLiquidationBidsByCollectionAddressList,
      cwQueryLiquidationBidsByCollectionAddress,
      cwQueryLiquidationBidsByLiquidatorAddressList,
      cwQueryLiquidationBidsByLiquidatorAddress,
      cwQueryProposals,
      cwQueryCollectionList,
      cwQueryCollection,
      cwQueryBglCurrencyToMainCurrencyPrice,
      cwQueryConditionalDepositApr,
      cwQueryLtvList,
      cwQueryConditionalLtv,
      cwQueryTotalAvailableToBorrowLiquidity,
      cwQueryAvailableToBorrow,
      cwQueryAmounts
      // cwQueryTargets,
    },
    minter: {
      cwQueryCurrenciesByCreator,
      cwQueryConfig: cwMinterQueryConfig
    },
    oracle: {
      cwQueryConfig: cwOracleQueryConfig,
      cwQueryPrices,
      cwQueryBlockTime
    }
  };
}
export { getCwExecHelpers, getCwQueryHelpers };