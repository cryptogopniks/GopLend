import { getLast, l } from "../utils";
import { toBase64, fromUtf8 } from "@cosmjs/encoding";
import { LendingPlatformMsgComposer } from "../codegen/LendingPlatform.message-composer";
import { OracleMsgComposer } from "../codegen/Oracle.message-composer";
import { LendingPlatformQueryClient } from "../codegen/LendingPlatform.client";
import { OracleQueryClient } from "../codegen/Oracle.client";
import CONFIG_JSON from "../config/config.json";
import { getChainOptionById, getContractByWasm } from "../config/config-utils";
import { RawPriceItem } from "../codegen/Oracle.types";
import {
  getCwClient,
  signAndBroadcastWrapper,
  getExecuteContractMsg,
} from "./clients";
import {
  SigningCosmWasmClient,
  CosmWasmClient,
  MsgExecuteContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";
import {
  CollectionInfoForString,
  ProposalForStringAndTokenUnverified,
  QueryLiquidationBidsResponseItem,
  QueryMsg as LendingPlatformQueryMsg,
} from "../codegen/LendingPlatform.types";
import {
  DirectSecp256k1HdWallet,
  OfflineSigner,
  OfflineDirectSigner,
  coin,
} from "@cosmjs/proto-signing";
import {
  UpdateConfigStruct,
  Cw20SendMsg,
  TokenUnverified,
  ChainConfig,
  ContractInfo,
  QueryAllOperatorsResponse,
  QueryAllOperatorsMsg,
  ApproveAllMsg,
  RevokeAllMsg,
  QueryApprovalsMsg,
  ApprovalsResponse,
  QueryTokens,
  TokensResponse,
  QueryOwnerOf,
  OwnerOfResponse,
} from "../interfaces";

function addSingleTokenToComposerObj(
  obj: MsgExecuteContractEncodeObject,
  amount: number,
  token: TokenUnverified
): MsgExecuteContractEncodeObject {
  const {
    value: { contract, sender, msg },
  } = obj;

  if (!(contract && sender && msg)) {
    throw new Error(`${msg} parameters error!`);
  }

  return getSingleTokenExecMsg(
    contract,
    sender,
    JSON.parse(fromUtf8(msg)),
    amount,
    token
  );
}

function getSingleTokenExecMsg(
  contractAddress: string,
  senderAddress: string,
  msg: any,
  amount?: number,
  token?: TokenUnverified
) {
  // get msg without funds
  if (!(token && amount)) {
    return getExecuteContractMsg(contractAddress, senderAddress, msg, []);
  }

  // get msg with native token
  if ("native" in token) {
    return getExecuteContractMsg(contractAddress, senderAddress, msg, [
      coin(amount, token.native.denom),
    ]);
  }

  // get msg with CW20 token
  const cw20SendMsg: Cw20SendMsg = {
    send: {
      contract: contractAddress,
      amount: `${amount}`,
      msg: toBase64(msg),
    },
  };

  return getExecuteContractMsg(
    token.cw20.address,
    senderAddress,
    cw20SendMsg,
    []
  );
}

function getApproveCollectionMsg(
  collectionAddress: string,
  senderAddress: string,
  operator: string
): MsgExecuteContractEncodeObject {
  const approveAllMsg: ApproveAllMsg = {
    approve_all: {
      operator,
    },
  };

  return getSingleTokenExecMsg(collectionAddress, senderAddress, approveAllMsg);
}

function getRevokeCollectionMsg(
  collectionAddress: string,
  senderAddress: string,
  operator: string
): MsgExecuteContractEncodeObject {
  const revokeAllMsg: RevokeAllMsg = {
    revoke_all: {
      operator,
    },
  };

  return getSingleTokenExecMsg(collectionAddress, senderAddress, revokeAllMsg);
}

async function getCwExecHelpers(
  chainId: string,
  rpc: string,
  owner: string,
  signer: (OfflineSigner & OfflineDirectSigner) | DirectSecp256k1HdWallet
) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  let LENDING_PLATFORM_CONTRACT: ContractInfo | undefined;
  let ORACLE_CONTRACT: ContractInfo | undefined;

  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(
      CONTRACTS,
      "lending_platform.wasm"
    );
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

  const signingClient = cwClient.client as SigningCosmWasmClient;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

  const landingPlatformMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : ""
  );

  const oracleMsgComposer = new OracleMsgComposer(
    owner,
    ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : ""
  );

  async function _msgWrapperWithGasPrice(
    msgs: MsgExecuteContractEncodeObject[],
    gasPrice: string,
    gasAdjustment: number = 1,
    memo?: string
  ) {
    const tx = await _signAndBroadcast(msgs, gasPrice, gasAdjustment, memo);
    l("\n", tx, "\n");
    return tx;
  }

  // utils

  async function cwRevoke(
    collectionAddress: string,
    senderAddress: string,
    operator: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getRevokeCollectionMsg(collectionAddress, senderAddress, operator)],
      gasPrice
    );
  }

  // lending-platform

  async function cwDeposit(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          landingPlatformMsgComposer.deposit(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwUnbond(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          landingPlatformMsgComposer.unbond(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdraw(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          landingPlatformMsgComposer.withdraw(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwApproveAndDepositCollateral(
    senderAddress: string,
    operator: string,
    collections: CollectionInfoForString[],
    gasPrice: string
  ) {
    const queryAllOperatorsMsg: QueryAllOperatorsMsg = {
      all_operators: {
        owner: senderAddress,
      },
    };

    let msgList: MsgExecuteContractEncodeObject[] = [];

    for (const { collection_address: collectionAddress } of collections) {
      const { operators }: QueryAllOperatorsResponse =
        await signingClient.queryContractSmart(
          collectionAddress,
          queryAllOperatorsMsg
        );

      const targetOperator = operators.find((x) => x.spender === operator);

      if (!targetOperator) {
        msgList.push(
          getApproveCollectionMsg(collectionAddress, senderAddress, operator)
        );
      }
    }

    msgList.push(landingPlatformMsgComposer.deposit());

    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }

  async function cwWithdrawCollateral(
    collections: CollectionInfoForString[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.withdrawCollateral({ collections })],
      gasPrice,
      1.05
    );
  }

  async function cwBorrow(amount: number, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.borrow({ amount: `${amount}` })],
      gasPrice
    );
  }

  async function cwRepay(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          landingPlatformMsgComposer.repay(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwPlaceBid(
    collections: CollectionInfoForString[],
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          landingPlatformMsgComposer.placeBid({ collections }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwRemoveBid(
    collections: CollectionInfoForString[],
    creationDate: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.removeBid({ collections, creationDate })],
      gasPrice
    );
  }

  async function cwUpdateBid(
    collections: CollectionInfoForString[],
    creationDate: number,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    // query existing bids
    const queryBidsMsg: LendingPlatformQueryMsg = {
      query_liquidation_bids_by_liquidator_addresses: { addresses: [owner] },
    };

    const bids: QueryLiquidationBidsResponseItem[] =
      await signingClient.queryContractSmart(
        LENDING_PLATFORM_CONTRACT?.ADDRESS || "",
        queryBidsMsg
      );

    const currentCollectionsAddresses = collections.map(
      (x) => x.collection_address
    );
    const bidsForCurrentCollections = bids.filter((x) =>
      currentCollectionsAddresses.includes(x.address)
    );

    // calculate funds to send
    let amountToSend = 0;

    for (const { liquidation_bids } of bidsForCurrentCollections) {
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
      amount: `${amount}`,
    });

    if (amountToSend > 0) {
      return await _msgWrapperWithGasPrice(
        [addSingleTokenToComposerObj(msgObj, amountToSend, token)],
        gasPrice
      );
    }

    return await _msgWrapperWithGasPrice([msgObj], gasPrice);
  }

  async function cwClaimCollateral(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.claimCollateral()],
      gasPrice
    );
  }

  async function cwProcess(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.process()],
      gasPrice
    );
  }

  async function cwCreateBglToken(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.createBglToken()],
      gasPrice
    );
  }

  async function cwLock(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.lock()],
      gasPrice
    );
  }

  async function cwUnlock(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.unlock()],
      gasPrice
    );
  }

  async function cwDistributeFunds(
    addressAndWeightList: [string, string][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.distributeFunds({ addressAndWeightList })],
      gasPrice
    );
  }

  async function cwRemoveCollection(address: string, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.removeCollection({ address })],
      gasPrice
    );
  }

  async function cwCreateProposal(
    proposal: ProposalForStringAndTokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.createProposal({ proposal })],
      gasPrice
    );
  }

  async function cwRejectProposal(id: number, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [landingPlatformMsgComposer.rejectProposal({ id })],
      gasPrice
    );
  }

  async function cwAcceptProposal(
    id: number,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          landingPlatformMsgComposer.acceptProposal({ id }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  // oracle

  async function cwUpdatePrices(data: RawPriceItem[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [oracleMsgComposer.updatePrices({ data })],
      gasPrice
    );
  }

  async function cwUpdateConfig(
    updateConfigStruct: UpdateConfigStruct,
    gasPrice: string
  ) {
    let msgList: MsgExecuteContractEncodeObject[] = [];

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
      schedulerForOracle,
    } = updateConfigStruct;

    if (schedulerForOracle) {
      msgList.push(
        oracleMsgComposer.updateConfig({ scheduler: schedulerForOracle })
      );
    }

    if (
      bglCurrency ||
      bidMinRate ||
      borrowApr ||
      borrowFeeRate ||
      cw20BaseCodeId ||
      discountMaxRate ||
      discountMinRate ||
      liquidationFeeRate ||
      mainCurrency ||
      oracle ||
      owner ||
      schedulerForLendingPlatform ||
      unbondingPeriod
    ) {
      msgList.push(
        landingPlatformMsgComposer.updateConfig({
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
          unbondingPeriod,
        })
      );
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

    cwRevoke,
  };
}

async function getCwQueryHelpers(chainId: string, rpc: string) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  let LENDING_PLATFORM_CONTRACT: ContractInfo | undefined;
  let ORACLE_CONTRACT: ContractInfo | undefined;

  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(
      CONTRACTS,
      "lending_platform.wasm"
    );
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

  const cosmwasmQueryClient: CosmWasmClient = cwClient.client;

  const lendingPlatformQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : ""
  );

  const oracleQueryClient = new OracleQueryClient(
    cosmwasmQueryClient,
    ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : ""
  );

  // utils

  async function cwQueryOperators(
    collectionAddress: string,
    ownerAddress: string
  ) {
    const queryAllOperatorsMsg: QueryAllOperatorsMsg = {
      all_operators: {
        owner: ownerAddress,
      },
    };
    const res: QueryAllOperatorsResponse =
      await cosmwasmQueryClient.queryContractSmart(
        collectionAddress,
        queryAllOperatorsMsg
      );
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryApprovals(collectionAddress: string, tokenId: string) {
    const queryApprovalsMsg: QueryApprovalsMsg = {
      approvals: {
        token_id: tokenId,
      },
    };
    const res: ApprovalsResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryApprovalsMsg
    );
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryBalanceInNft(owner: string, collectionAddress: string) {
    const MAX_LIMIT = 100;
    const ITER_LIMIT = 50;

    let tokenList: string[] = [];
    let tokenAmountSum: number = 0;
    let i: number = 0;
    let lastToken: string | undefined = undefined;

    while ((!i || tokenAmountSum === MAX_LIMIT) && i < ITER_LIMIT) {
      i++;

      try {
        const queryTokensMsg: QueryTokens = {
          tokens: {
            owner,
            start_after: lastToken,
            limit: MAX_LIMIT,
          },
        };

        const { tokens }: TokensResponse =
          await cosmwasmQueryClient.queryContractSmart(
            collectionAddress,
            queryTokensMsg
          );

        tokenList = [...tokenList, ...tokens];
        tokenAmountSum = tokens.length;
        lastToken = getLast(tokens);
      } catch (error) {
        l(error);
      }
    }

    const res: TokensResponse = { tokens: tokenList };
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryNftOwner(collectionAddress: string, tokenId: string) {
    const queryOwnerOfMsg: QueryOwnerOf = {
      owner_of: { token_id: tokenId },
    };
    const res: OwnerOfResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryOwnerOfMsg
    );
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

  async function cwQueryDepositors(addresses?: string[]) {
    const res = await lendingPlatformQueryClient.queryDepositors({ addresses });
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryBorrowers(addresses?: string[]) {
    const res = await lendingPlatformQueryClient.queryBorrowers({ addresses });
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryLiquidators(addresses?: string[]) {
    const res = await lendingPlatformQueryClient.queryLiquidators({
      addresses,
    });
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryCollaterals(collectionAddresses?: string[]) {
    const res = await lendingPlatformQueryClient.queryCollaterals({
      collectionAddresses,
    });
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryLiquidationBidsByCollectionAddresses(
    addresses?: string[]
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddresses(
        { addresses }
      );
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryLiquidationBidsByLiquidatorAddresses(
    addresses?: string[]
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddresses(
        { addresses }
      );
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryProposals(lastAmount?: number) {
    const res = await lendingPlatformQueryClient.queryProposals({
      lastAmount,
    });
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryCollections(addresses?: string[]) {
    const res = await lendingPlatformQueryClient.queryCollections({
      addresses,
    });
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryLendingPlatformConfig() {
    const res = await lendingPlatformQueryClient.queryConfig();
    l("\n", res, "\n");
    return res;
  }

  async function cwQueryBlockCounter() {
    const res = await lendingPlatformQueryClient.queryBlockCounter();
    l("\n", res, "\n");
    return res;
  }

  // oracle

  async function cwQueryPrices(collectionAddresses?: string[]) {
    const res = await oracleQueryClient.queryPrices({ collectionAddresses });
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
    cwQueryOracleConfig,

    cwQueryBlockCounter,
  };
}

export { getCwExecHelpers, getCwQueryHelpers };
