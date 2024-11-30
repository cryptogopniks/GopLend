import { SchedulerMsgComposer } from "../codegen/Scheduler.message-composer";
import { SchedulerQueryClient } from "../codegen/Scheduler.client";

import { LendingPlatformMsgComposer } from "../codegen/LendingPlatform.message-composer";
import { LendingPlatformQueryClient } from "../codegen/LendingPlatform.client";

import { MinterMsgComposer } from "../codegen/Minter.message-composer";
import { MinterQueryClient } from "../codegen/Minter.client";

import { OracleMsgComposer } from "../codegen/Oracle.message-composer";
import { OracleQueryClient } from "../codegen/Oracle.client";

import { MarketMakerMsgComposer } from "../codegen/MarketMaker.message-composer";
import { MarketMakerQueryClient } from "../codegen/MarketMaker.client";

import CONFIG_JSON from "../config/config.json";
import { getLast, getPaginationAmount, l, li, logAndReturn } from "../utils";
import { toBase64, fromUtf8, toUtf8 } from "@cosmjs/encoding";
import {
  MsgMigrateContract,
  MsgUpdateAdmin,
} from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { getChainOptionById, getContractByLabel } from "../config/config-utils";
import {
  getCwClient,
  signAndBroadcastWrapper,
  getExecuteContractMsg,
} from "./clients";
import {
  SigningCosmWasmClient,
  CosmWasmClient,
  MsgExecuteContractEncodeObject,
  MsgUpdateAdminEncodeObject,
  MsgMigrateContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";
import {
  ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
  BiddedCollateralItem,
  CollectionInfoForString,
  CurrencyForTokenUnverified,
  LiquidationItem,
  ProposalForStringAndTokenUnverified,
  QueryBorrowersResponseItem,
  QueryCollateralsResponseItem,
  QueryCollectionsResponseItem,
  QueryLiquidationBidsByCollectionAddressListResponseItem,
  QueryLiquidatorsResponseItem,
  QueryMsg as LendingPlatformQueryMsg,
  QueryProposalsResponseItem,
  QueryUnbondersResponseItem,
} from "../codegen/LendingPlatform.types";
import {
  DirectSecp256k1HdWallet,
  OfflineSigner,
  OfflineDirectSigner,
  coin,
} from "@cosmjs/proto-signing";
import {
  InstantiateMarketingInfo,
  Logo,
  Metadata,
} from "../codegen/Minter.types";
import {
  CollateralListResponseItem,
  CollectionOwnerForAddr,
  LiquidityInfo,
  OffersResponse,
} from "../codegen/MarketMaker.types";
import * as v2 from "../interfaces/stargaze-marketplace-v2";
import {
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
import { PriceItem, QueryPricesResponse } from "../codegen/Oracle.types";

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

function getContracts(contracts: ContractInfo[]) {
  let SCHEDULER_CONTRACT: ContractInfo | undefined;
  let LENDING_PLATFORM_CONTRACT: ContractInfo | undefined;
  let MINTER_CONTRACT: ContractInfo | undefined;
  let ORACLE_CONTRACT: ContractInfo | undefined;
  let MARKET_MAKER_CONTRACT: ContractInfo | undefined;

  try {
    SCHEDULER_CONTRACT = getContractByLabel(contracts, "scheduler");
  } catch (error) {
    l(error);
  }

  try {
    LENDING_PLATFORM_CONTRACT = getContractByLabel(
      contracts,
      "lending_platform"
    );
  } catch (error) {
    l(error);
  }

  try {
    MINTER_CONTRACT = getContractByLabel(contracts, "minter");
  } catch (error) {
    l(error);
  }

  try {
    ORACLE_CONTRACT = getContractByLabel(contracts, "oracle");
  } catch (error) {
    l(error);
  }

  try {
    MARKET_MAKER_CONTRACT = getContractByLabel(contracts, "market_maker");
  } catch (error) {
    l(error);
  }

  return {
    SCHEDULER_CONTRACT,
    LENDING_PLATFORM_CONTRACT,
    MINTER_CONTRACT,
    ORACLE_CONTRACT,
    MARKET_MAKER_CONTRACT,
  };
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

  const {
    SCHEDULER_CONTRACT,
    LENDING_PLATFORM_CONTRACT,
    MINTER_CONTRACT,
    ORACLE_CONTRACT,
    MARKET_MAKER_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc, owner, signer);
  if (!cwClient) throw new Error("cwClient is not found!");

  const signingClient = cwClient.client as SigningCosmWasmClient;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

  const schedulerMsgComposer = new SchedulerMsgComposer(
    owner,
    SCHEDULER_CONTRACT?.ADDRESS || ""
  );

  const lendingPlatformMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_CONTRACT?.ADDRESS || ""
  );

  const minterMsgComposer = new MinterMsgComposer(
    owner,
    MINTER_CONTRACT?.ADDRESS || ""
  );

  const oracleMsgComposer = new OracleMsgComposer(
    owner,
    ORACLE_CONTRACT?.ADDRESS || ""
  );

  const marketMakerMsgComposer = new MarketMakerMsgComposer(
    owner,
    MARKET_MAKER_CONTRACT?.ADDRESS || ""
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

  async function cwTransferAdmin(
    contract: string,
    newAdmin: string,
    gasPrice: string,
    gasAdjustment: number = 1
  ) {
    const msg: MsgUpdateAdminEncodeObject = {
      typeUrl: "/cosmwasm.wasm.v1.MsgUpdateAdmin",
      value: MsgUpdateAdmin.fromPartial({
        sender: owner,
        contract,
        newAdmin,
      }),
    };

    const tx = await _signAndBroadcast([msg], gasPrice, gasAdjustment);
    l("\n", tx, "\n");
    return tx;
  }

  async function cwMigrateMultipleContracts(
    contractList: string[],
    codeId: number,
    migrateMsg: any,
    gasPrice: string,
    gasAdjustment: number = 1
  ) {
    const msgList: MsgMigrateContractEncodeObject[] = contractList.map(
      (contract) => ({
        typeUrl: "/cosmwasm.wasm.v1.MsgMigrateContract",
        value: MsgMigrateContract.fromPartial({
          sender: owner,
          contract,
          codeId: BigInt(codeId),
          msg: toUtf8(JSON.stringify(migrateMsg)),
        }),
      })
    );

    const tx = await _signAndBroadcast(msgList, gasPrice, gasAdjustment);
    l("\n", tx, "\n");
    return tx;
  }

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

  async function cwMintNft(
    collectionAddress: string,
    recipient: string,
    tokenIdList: number[],
    gasPrice: string
  ) {
    const msgList = tokenIdList.map((tokenId) => {
      const mintMsg = {
        mint: {
          owner: recipient,
          token_id: tokenId.toString(),
        },
      };

      return getSingleTokenExecMsg(collectionAddress, owner, mintMsg);
    });

    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }

  // scheduler

  async function cwAdapterSchedulerCommonAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [schedulerMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwAdapterSchedulerCommonUpdateConfig(
    {
      admin,
      worker,
      lendingPlatform,
      executionCooldown,
      offchainClock,
    }: {
      admin?: string;
      worker?: string;
      lendingPlatform?: string;
      executionCooldown?: number;
      offchainClock?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        schedulerMsgComposer.updateConfig({
          admin,
          worker,
          lendingPlatform,
          offchainClock,
          executionCooldown,
        }),
      ],
      gasPrice
    );
  }

  async function cwPush(targets: LiquidationItem[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [schedulerMsgComposer.push({ targets })],
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
          lendingPlatformMsgComposer.deposit(),
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
          lendingPlatformMsgComposer.unbond(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdraw(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.withdraw()],
      gasPrice
    );
  }

  async function cwWithdrawCollateral(
    collections: CollectionInfoForString[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.withdrawCollateral({ collections })],
      gasPrice,
      1.05
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

    msgList.push(lendingPlatformMsgComposer.depositCollateral({ collections }));

    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }

  async function cwBorrow(amount: number, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.borrow({ amount: `${amount}` })],
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
          lendingPlatformMsgComposer.repay(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwPlaceBid(
    collections: CollectionInfoForString[],
    discount: number,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          lendingPlatformMsgComposer.placeBid({
            collections,
            discount: discount.toString(),
          }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwRemoveBid(
    collectionAddresses: string[],
    creationDate: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.removeBid({
          collectionAddresses,
          creationDate,
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdateBid(
    collections: CollectionInfoForString[],
    creationDate: number,
    amount: number,
    discount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    // query existing bids
    const queryBidsMsg: LendingPlatformQueryMsg = {
      query_liquidation_bids_by_liquidator_address: { address: owner },
    };

    const bids: ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem =
      await signingClient.queryContractSmart(
        LENDING_PLATFORM_CONTRACT?.ADDRESS || "",
        queryBidsMsg
      );

    const currentCollectionsAddresses = collections.map(
      (x) => x.collection_address
    );
    const bidsForCurrentCollections = bids.filter((x) =>
      currentCollectionsAddresses.includes(x.collection_address)
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

    const msgObj = lendingPlatformMsgComposer.updateBid({
      collections,
      creationDate,
      amount: amount.toString(),
      discount: discount.toString(),
    });

    if (amountToSend > 0) {
      return await _msgWrapperWithGasPrice(
        [addSingleTokenToComposerObj(msgObj, amountToSend, token)],
        gasPrice
      );
    }

    return await _msgWrapperWithGasPrice([msgObj], gasPrice);
  }

  async function cwLiquidate(targets: LiquidationItem[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.liquidate({ targets })],
      gasPrice
    );
  }

  async function cwLendingPlatformAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwLendingPlatformUpdateAddressConfig(
    {
      admin,
      worker,
      minter,
      oracle,
      scheduler,
      marketMaker,
    }: {
      admin?: string;
      worker?: string;
      minter?: string;
      oracle?: string;
      scheduler?: string;
      marketMaker?: string;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.updateAddressConfig({
          admin,
          worker,
          minter,
          oracle,
          scheduler,
          marketMaker,
        }),
      ],
      gasPrice
    );
  }

  async function cwLendingPlatformUpdateRateConfig(
    {
      bidMinRate,
      borrowApr,
      borrowFeeRate,
      discountMaxRate,
      discountMinRate,
      liquidationFeeRate,
    }: {
      bidMinRate?: number;
      borrowApr?: number;
      borrowFeeRate?: number;
      discountMaxRate?: number;
      discountMinRate?: number;
      liquidationFeeRate?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.updateRateConfig({
          bidMinRate: bidMinRate?.toString(),
          borrowApr: borrowApr?.toString(),
          borrowFeeRate: borrowFeeRate?.toString(),
          discountMaxRate: discountMaxRate?.toString(),
          discountMinRate: discountMinRate?.toString(),
          liquidationFeeRate: liquidationFeeRate?.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwLendingPlatformUpdateCommonConfig(
    {
      bglCurrency,
      collateralMinValue,
      mainCurrency,
      borrowersReserveFractionRatio,
      unbondingPeriod,
    }: {
      bglCurrency?: CurrencyForTokenUnverified;
      collateralMinValue?: number;
      mainCurrency?: CurrencyForTokenUnverified;
      borrowersReserveFractionRatio?: number;
      unbondingPeriod?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.updateCommonConfig({
          bglCurrency,
          collateralMinValue: collateralMinValue?.toString(),
          mainCurrency,
          borrowersReserveFractionRatio:
            borrowersReserveFractionRatio?.toString(),
          unbondingPeriod,
        }),
      ],
      gasPrice
    );
  }

  async function cwDepositReserveLiquidity(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          lendingPlatformMsgComposer.depositReserveLiquidity(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdrawReserveLiquidity(amount: number, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.withdrawReserveLiquidity({
          amount: amount.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwReinforceBglToken(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          lendingPlatformMsgComposer.reinforceBglToken(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwPause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.pause()],
      gasPrice
    );
  }

  async function cwUnpause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.unpause()],
      gasPrice
    );
  }

  async function cwDistributeFunds(
    addressAndWeightList: [string, number][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.distributeFunds({
          addressAndWeightList: addressAndWeightList.map(
            ([address, weight]) => [address, weight.toString()]
          ),
        }),
      ],
      gasPrice
    );
  }

  async function cwRemoveCollection(address: string, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.removeCollection({ address })],
      gasPrice
    );
  }

  async function cwCreateProposal(
    proposal: ProposalForStringAndTokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.createProposal({ proposal })],
      gasPrice
    );
  }

  async function cwRejectProposal(id: number, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.rejectProposal({ id })],
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
          lendingPlatformMsgComposer.acceptProposal({ id }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  // minter

  async function cwMinterAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwMinterAcceptTokenOwnerRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.acceptTokenOwnerRole()],
      gasPrice
    );
  }

  async function cwMinterPause(gasPrice: string) {
    return await _msgWrapperWithGasPrice([minterMsgComposer.pause()], gasPrice);
  }

  async function cwMinterUnpause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.unpause()],
      gasPrice
    );
  }

  async function cwMinterUpdateConfig(
    {
      admin,
      cw20CodeId,
      maxTokensPerOwner,
      permissionlessTokenCreation,
      permissionlessTokenRegistration,
      whitelist,
    }: {
      admin?: string;
      cw20CodeId?: number;
      maxTokensPerOwner?: number;
      permissionlessTokenCreation?: boolean;
      permissionlessTokenRegistration?: boolean;
      whitelist?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateConfig({
          admin,
          cw20CodeId,
          maxTokensPerOwner,
          permissionlessTokenCreation,
          permissionlessTokenRegistration,
          whitelist,
        }),
      ],
      gasPrice
    );
  }

  async function cwCreateNative(
    subdenom: string,
    {
      decimals,
      owner,
      permissionlessBurning,
      whitelist,
    }: {
      decimals?: number;
      owner?: string;
      permissionlessBurning?: boolean;
      whitelist?: string[];
    },
    paymentAmount: number,
    paymentDenom: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          minterMsgComposer.createNative({
            subdenom,
            decimals,
            owner,
            permissionlessBurning,
            whitelist,
          }),
          paymentAmount,
          {
            native: { denom: paymentDenom },
          }
        ),
      ],
      gasPrice
    );
  }

  async function cwCreateCw20(
    name: string,
    symbol: string,
    {
      cw20CodeId,
      decimals,
      marketing,
      owner,
      permissionlessBurning,
      whitelist,
    }: {
      cw20CodeId?: number;
      decimals?: number;
      marketing?: InstantiateMarketingInfo;
      owner?: string;
      permissionlessBurning?: boolean;
      whitelist?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.createCw20({
          name,
          symbol,
          cw20CodeId,
          decimals,
          marketing,
          owner,
          permissionlessBurning,
          whitelist,
        }),
      ],
      gasPrice
    );
  }

  async function cwRegisterNative(
    denom: string,
    {
      decimals,
      owner,
      permissionlessBurning,
      whitelist,
    }: {
      decimals?: number;
      owner?: string;
      permissionlessBurning?: boolean;
      whitelist?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.registerNative({
          denom,
          decimals,
          owner,
          permissionlessBurning,
          whitelist,
        }),
      ],
      gasPrice
    );
  }

  async function cwRegisterCw20(
    address: string,
    {
      cw20CodeId,
      decimals,
      owner,
      permissionlessBurning,
      whitelist,
    }: {
      cw20CodeId?: number;
      decimals?: number;
      owner?: string;
      permissionlessBurning?: boolean;
      whitelist?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.registerCw20({
          address,
          cw20CodeId,
          decimals,
          owner,
          permissionlessBurning,
          whitelist,
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdateCurrencyInfo(
    denomOrAddress: string,
    {
      owner,
      permissionlessBurning,
      whitelist,
    }: {
      owner?: string;
      permissionlessBurning?: boolean;
      whitelist?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateCurrencyInfo({
          denomOrAddress,
          owner,
          permissionlessBurning,
          whitelist,
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdateMetadataNative(
    denom: string,
    metadata: Metadata,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateMetadataNative({
          denom,
          metadata,
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdateMetadataCw20(
    address: string,
    {
      description,
      logo,
      project,
    }: {
      description?: string;
      logo?: Logo;
      project?: string;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateMetadataCw20({
          address,
          description,
          logo,
          project,
        }),
      ],
      gasPrice
    );
  }

  async function cwExcludeNative(denom: string, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.excludeNative({ denom })],
      gasPrice
    );
  }

  async function cwExcludeCw20(address: string, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.excludeCw20({ address })],
      gasPrice
    );
  }

  async function cwMint(
    amount: number,
    denomOrAddress: string,
    recipient: string | undefined = undefined,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.mint({
          denomOrAddress,
          amount: amount.toString(),
          recipient,
        }),
      ],
      gasPrice
    );
  }

  async function cwMintMultiple(
    denomOrAddress: string,
    accountAndAmountList: [string, number][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.mintMultiple({
          denomOrAddress,
          accountAndAmountList: accountAndAmountList.map(
            ([account, amount]) => [account, amount.toString()]
          ),
        }),
      ],
      gasPrice
    );
  }

  async function cwBurn(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [addSingleTokenToComposerObj(minterMsgComposer.burn(), amount, token)],
      gasPrice
    );
  }

  // oracle

  async function cwOracleAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [oracleMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwOracleUpdateConfig(
    {
      admin,
      worker,
      controller,
      maxPriceUpdatePeriod,
      executionCooldown,
    }: {
      admin?: string;
      worker?: string;
      controller?: string[];
      maxPriceUpdatePeriod?: number;
      executionCooldown?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        oracleMsgComposer.updateConfig({
          admin,
          worker,
          controller,
          maxPriceUpdatePeriod,
          executionCooldown,
        }),
      ],
      gasPrice
    );
  }

  // list of (collection_address_postfix, usd_price_with_decimals_2)
  async function cwUpdatePrices(data: [string, number][], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [
        oracleMsgComposer.updatePrices({
          data: data.map(([collectionAddress, usdPrice]) => {
            const postfix = collectionAddress.split("1").slice(1).join("1");
            // u32 was recognized as string for no reason
            const price = (usdPrice * 100) as any;

            return [postfix, price];
          }),
        }),
      ],
      gasPrice
    );
  }

  async function cwRemovePrices(collections: string[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [oracleMsgComposer.removePrices({ collections })],
      gasPrice
    );
  }

  // market-maker

  async function cwMarketMakerAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwMarketMakerUpdateConfig(
    {
      admin,
      worker,
      controller,
      lendingPlatform,
      oracle,
      token,
    }: {
      admin?: string;
      worker?: string;
      controller?: string[];
      lendingPlatform?: string;
      oracle?: string;
      token?: TokenUnverified;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerMsgComposer.updateConfig({
          admin,
          worker,
          controller,
          lendingPlatform,
          oracle,
          token,
        }),
      ],
      gasPrice
    );
  }

  async function cwMarketMakerPause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerMsgComposer.pause()],
      gasPrice
    );
  }

  async function cwMarketMakerUnpause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerMsgComposer.unpause()],
      gasPrice
    );
  }

  async function cwSetCollection(
    collection: string,
    owner: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerMsgComposer.setCollection({
          collection,
          owner,
        }),
      ],
      gasPrice
    );
  }

  async function cwRemoveCollections(
    collectionList: string[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [marketMakerMsgComposer.removeCollections({ collectionList })],
      gasPrice
    );
  }

  async function cwDepositLiquidity(
    collection: string,
    amount: number,
    denom: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          marketMakerMsgComposer.depositLiquidity({ collection }),
          amount,
          {
            native: { denom },
          }
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdrawUndistributedLiquidity(
    collection: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerMsgComposer.withdrawUndistributedLiquidity({
          collection,
        }),
      ],
      gasPrice
    );
  }

  async function cwMarketMakerWithdrawCollateral(
    collection: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [marketMakerMsgComposer.withdrawCollateral({ collection })],
      gasPrice
    );
  }

  async function cwUpdateOffers(
    collection: string,
    fromToPriceList: [number, number][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerMsgComposer.updateOffers({
          collection,
          fromToPriceList: fromToPriceList.map(([priceBefore, priceAfter]) => [
            priceBefore.toString(),
            priceAfter.toString(),
          ]),
        }),
      ],
      gasPrice
    );
  }

  return {
    utils: { cwTransferAdmin, cwMigrateMultipleContracts, cwRevoke, cwMintNft },
    scheduler: {
      cwAcceptAdminRole: cwAdapterSchedulerCommonAcceptAdminRole,
      cwUpdateConfig: cwAdapterSchedulerCommonUpdateConfig,
      cwPush,
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
      cwAcceptAdminRole: cwLendingPlatformAcceptAdminRole,
      cwUpdateAddressConfig: cwLendingPlatformUpdateAddressConfig,
      cwUpdateRateConfig: cwLendingPlatformUpdateRateConfig,
      cwUpdateCommonConfig: cwLendingPlatformUpdateCommonConfig,
      cwDepositReserveLiquidity,
      cwWithdrawReserveLiquidity,
      cwReinforceBglToken,
      cwPause,
      cwUnpause,
      cwDistributeFunds,
      cwRemoveCollection,
      cwCreateProposal,
      cwRejectProposal,
      cwAcceptProposal,
    },
    minter: {
      cwAcceptAdminRole: cwMinterAcceptAdminRole,
      cwAcceptTokenOwnerRole: cwMinterAcceptTokenOwnerRole,
      cwMinterPause: cwMinterPause,
      cwMinterUnpause: cwMinterUnpause,
      cwUpdateConfig: cwMinterUpdateConfig,
      cwCreateNative,
      cwCreateCw20,
      cwRegisterNative,
      cwRegisterCw20,
      cwUpdateCurrencyInfo,
      cwUpdateMetadataNative,
      cwUpdateMetadataCw20,
      cwExcludeNative,
      cwExcludeCw20,
      cwMint,
      cwMintMultiple,
      cwBurn,
    },
    oracle: {
      cwAcceptAdminRole: cwOracleAcceptAdminRole,
      cwUpdateConfig: cwOracleUpdateConfig,
      cwUpdatePrices,
      cwRemovePrices,
    },
    marketMaker: {
      cwAcceptAdminRole: cwMarketMakerAcceptAdminRole,
      cwUpdateConfig: cwMarketMakerUpdateConfig,
      cwPause: cwMarketMakerPause,
      cwUnpause: cwMarketMakerUnpause,
      cwSetCollection,
      cwRemoveCollections,
      cwDepositLiquidity,
      cwWithdrawUndistributedLiquidity,
      cwWithdrawCollateral: cwMarketMakerWithdrawCollateral,
      cwUpdateOffers,
    },
  };
}

async function getCwQueryHelpers(chainId: string, rpc: string) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const {
    SCHEDULER_CONTRACT,
    LENDING_PLATFORM_CONTRACT,
    MINTER_CONTRACT,
    ORACLE_CONTRACT,
    MARKET_MAKER_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc);
  if (!cwClient) throw new Error("cwClient is not found!");

  const cosmwasmQueryClient: CosmWasmClient = cwClient.client;

  const schedulerQueryClient = new SchedulerQueryClient(
    cosmwasmQueryClient,
    SCHEDULER_CONTRACT?.ADDRESS || ""
  );

  const lendingPlatformQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_CONTRACT?.ADDRESS || ""
  );

  const minterQueryClient = new MinterQueryClient(
    cosmwasmQueryClient,
    MINTER_CONTRACT?.ADDRESS || ""
  );

  const oracleQueryClient = new OracleQueryClient(
    cosmwasmQueryClient,
    ORACLE_CONTRACT?.ADDRESS || ""
  );

  const marketMakerQueryClient = new MarketMakerQueryClient(
    cosmwasmQueryClient,
    MARKET_MAKER_CONTRACT?.ADDRESS || ""
  );

  // utils

  async function cwQueryOperators(
    ownerAddress: string,
    collectionAddress: string,
    isDisplayed: boolean = false
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
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryApprovals(
    collectionAddress: string,
    tokenId: string,
    isDisplayed: boolean = false
  ) {
    const queryApprovalsMsg: QueryApprovalsMsg = {
      approvals: {
        token_id: tokenId,
      },
    };
    const res: ApprovalsResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryApprovalsMsg
    );
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBalanceInNft(
    owner: string,
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
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
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryNftOwner(
    collectionAddress: string,
    tokenId: string,
    isDisplayed: boolean = false
  ) {
    const queryOwnerOfMsg: QueryOwnerOf = {
      owner_of: { token_id: tokenId },
    };
    const res: OwnerOfResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryOwnerOfMsg
    );
    return logAndReturn(res, isDisplayed);
  }

  // async function pQueryMarketplaceTokenOffers(
  //   marketplace: string,
  //   collection: string,
  //   blockTime: number,
  //   maxPaginationAmount: number,
  //   maxCount: number = 0,
  //   isDisplayed: boolean = false
  // ) {
  //   const defaultItem: BidOffset = { bidder: "", price: "", token_id: 0 };

  //   let allItems: Bid[] = [];
  //   let lastItem: BidOffset | undefined = undefined;
  //   let count: number = 0;

  //   while (
  //     JSON.stringify(lastItem) !== JSON.stringify(defaultItem) &&
  //     count < (maxCount || count + 1)
  //   ) {
  //     const queryMsg: v2.QueryBidsByTokenPrice = {
  //       bids_by_token_price: {
  //         collection,
  //         limit: maxPaginationAmount,
  //         start_before: lastItem,
  //       }
  //     };

  //     const res: BidsResponse = await cosmwasmQueryClient.queryContractSmart(
  //       marketplace,
  //       queryMsg
  //     );

  //     const { bidder, price, token_id } = getLast(res.bids) || defaultItem;
  //     lastItem = { bidder, price, token_id };
  //     const items = res.bids.filter(
  //       (x) => Math.floor(Number(x.expires_at) / 1e9) > blockTime
  //     );
  //     allItems = [...allItems, ...items];
  //     count += items.length;
  //   }

  //   if (maxCount) {
  //     allItems = allItems.slice(0, maxCount);
  //   }

  //   return logAndReturn(allItems, isDisplayed);
  // }

  // async function pQueryMarketplaceCollectionOffers(
  //   marketplace: string,
  //   collection: string,
  //   blockTime: number,
  //   maxPaginationAmount: number,
  //   maxCount: number = 0,
  //   isDisplayed: boolean = false
  // ) {
  //   const defaultItem: CollectionBidOffset = {
  //     bidder: "",
  //     price: "",
  //     collection: "",
  //   };
  //   let allItems: CollectionBid[] = [];
  //   let lastItem: CollectionBidOffset | undefined = undefined;
  //   let count: number = 0;

  //   while (
  //     JSON.stringify(lastItem) !== JSON.stringify(defaultItem) &&
  //     count < (maxCount || count + 1)
  //   ) {
  //     const queryMsg: QueryReverseCollectionBidsSortedByPrice = {
  //       reverse_collection_bids_sorted_by_price: {
  //         collection,
  //         limit: maxPaginationAmount,
  //         start_before: lastItem,
  //       },
  //     };

  //     const res: CollectionBidsResponse =
  //       await cosmwasmQueryClient.queryContractSmart(marketplace, queryMsg);

  //     const {
  //       bidder,
  //       price,
  //       collection: lastCollection,
  //     } = getLast(res.bids) || defaultItem;
  //     lastItem = { bidder, price, collection: lastCollection };
  //     const items = res.bids.filter(
  //       (x) => Math.floor(Number(x.expires_at) / 1e9) > blockTime
  //     );
  //     allItems = [...allItems, ...items];
  //     count += items.length;
  //   }

  //   if (maxCount) {
  //     allItems = allItems.slice(0, maxCount);
  //   }

  //   return logAndReturn(allItems, isDisplayed);
  // }

  async function cwGetMarketMakerData(
    collectionList: string[],
    batchPaginationAmount: number
  ): Promise<[string, string[]][]> {
    let promiseList: Promise<void>[] = [];
    let collectionOfferList: [string, string[]][] = [];

    const fn = async (collection: string) => {
      try {
        const { price_list } = await cwQueryOfferPrices(collection);

        if (price_list.length) {
          collectionOfferList.push([collection, price_list]);
        }
      } catch (error) {
        l(error);
      }
    };

    for (let i = 0; i < collectionList.length; i++) {
      if (promiseList.length >= batchPaginationAmount) {
        await Promise.all(promiseList);
        promiseList = [];
      }

      promiseList.push(fn(collectionList[i]));
    }

    await Promise.all(promiseList);

    return collectionOfferList;
  }

  // v2 queries

  async function cwV2QueryAsksByCollectionDenom(
    marketplace: string,
    collection: string,
    denom: string,
    queryOptions: v2.QueryOptions<v2.PriceOffset> | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const queryMsg: v2.QueryAsksByCollectionDenom = {
      asks_by_collection_denom: {
        collection,
        denom,
        query_options: queryOptions,
      },
    };

    const res: v2.Ask[] = await cosmwasmQueryClient.queryContractSmart(
      marketplace,
      queryMsg
    );

    return logAndReturn(res, isDisplayed);
  }

  async function cwV2QueryBidsByTokenPrice(
    marketplace: string,
    collection: string,
    token_id: string,
    denom: string,
    queryOptions: v2.QueryOptions<v2.PriceOffset> | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const queryMsg: v2.QueryBidsByTokenPrice = {
      bids_by_token_price: {
        collection,
        token_id,
        denom,
        query_options: queryOptions,
      },
    };

    const res: v2.Bid[] = await cosmwasmQueryClient.queryContractSmart(
      marketplace,
      queryMsg
    );

    return logAndReturn(res, isDisplayed);
  }

  async function cwV2QueryBidsByCreatorCollection(
    marketplace: string,
    collection: string,
    creator: string,
    queryOptions: v2.QueryOptions<string> | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const queryMsg: v2.QueryBidsByCreatorCollection = {
      bids_by_creator_collection: {
        collection,
        creator,
        query_options: queryOptions,
      },
    };

    const res: v2.Bid[] = await cosmwasmQueryClient.queryContractSmart(
      marketplace,
      queryMsg
    );

    return logAndReturn(res, isDisplayed);
  }

  // returns active bids in ascending (by default) price order
  async function cwV2QueryCollectionBidsByPrice(
    marketplace: string,
    collection: string,
    denom: string,
    queryOptions: v2.QueryOptions<v2.PriceOffset> | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const queryMsg: v2.QueryCollectionBidsByPrice = {
      collection_bids_by_price: {
        collection,
        denom,
        query_options: queryOptions,
      },
    };

    const res: v2.CollectionBid[] =
      await cosmwasmQueryClient.queryContractSmart(marketplace, queryMsg);

    return logAndReturn(res, isDisplayed);
  }

  async function cwV2QueryCollectionBid(
    marketplace: string,
    order_id: v2.OrderId,
    isDisplayed: boolean = false
  ) {
    const queryMsg: v2.QueryCollectionBid = {
      collection_bid: order_id,
    };

    const res: v2.QueryCollectionBidResponse =
      await cosmwasmQueryClient.queryContractSmart(marketplace, queryMsg);

    return logAndReturn(res, isDisplayed);
  }

  // scheduler

  async function cwAdapterSchedulerCommonQueryConfig(
    isDisplayed: boolean = false
  ) {
    const res = await schedulerQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwAdapterSchedulerCommonQueryLog(
    isDisplayed: boolean = false
  ) {
    const res = await schedulerQueryClient.queryLog();
    return logAndReturn(res, isDisplayed);
  }

  // lending-platform

  async function cwLendingPlatformQueryAddressConfig(
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryAddressConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwLendingPlatformQueryRateConfig(
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryRateConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwLendingPlatformQueryCommonConfig(
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryCommonConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryPlatformRevenue(isDisplayed: boolean = false) {
    const res = await lendingPlatformQueryClient.queryPlatformRevenue();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBalances(isDisplayed: boolean = false) {
    const res = await lendingPlatformQueryClient.queryBalances();
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryUnbonderList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryUnbondersResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryUnbondersResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryUnbondersResponseItem[] =
        await lendingPlatformQueryClient.queryUnbonderList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryUnbonder(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryUnbonder({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryBorrowerList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryBorrowersResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryBorrowersResponseItem[] =
        await lendingPlatformQueryClient.queryBorrowerList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryBorrower(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryBorrower({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryLiquidatorList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryLiquidatorsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryLiquidatorsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryLiquidatorsResponseItem[] =
        await lendingPlatformQueryClient.queryLiquidatorList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryLiquidator(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryLiquidator({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryCollateralList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryCollateralsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryCollateralsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryCollateralsResponseItem[] =
        await lendingPlatformQueryClient.queryCollateralList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryCollateral(
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryCollateral({
      collectionAddress,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCollateralByOwner(
    owner: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryCollateralByOwner({
      owner,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryLiquidationBidsByCollectionAddressList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryLiquidationBidsByCollectionAddressListResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
      [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
        await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddressList(
          {
            amount: paginationAmount,
            startAfter: lastItem,
          }
        );

      lastItem = getLast(items)?.collection_address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryLiquidationBidsByCollectionAddress(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddress({
        address,
      });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLiquidationBidsByLiquidatorAddressList(
    amount: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddressList(
        { startAfter, amount }
      );
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLiquidationBidsByLiquidatorAddress(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddress({
        address,
      });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryProposals(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryProposalsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryProposalsResponseItem[] = [];
    let firstItem: number | undefined = undefined;
    let count: number = 0;

    while (firstItem !== 0 && count < (maxCount || count + 1)) {
      const items: QueryProposalsResponseItem[] =
        await lendingPlatformQueryClient.queryProposals({
          amount: paginationAmount,
          startAfter: firstItem,
        });

      firstItem = getLast(items)?.id || 0;
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function pQueryCollectionList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryCollectionsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryCollectionsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryCollectionsResponseItem[] =
        await lendingPlatformQueryClient.queryCollectionList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryCollection(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryCollection({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBglCurrencyToMainCurrencyPrice(
    isDisplayed: boolean = false
  ) {
    const res =
      await lendingPlatformQueryClient.queryBglCurrencyToMainCurrencyPrice();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryConditionalDepositApr(
    amountToDeposit: number,
    amountToWithdraw: number,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryConditionalDepositApr({
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
      amountToWithdraw: amountToWithdraw
        ? amountToWithdraw.toString()
        : undefined,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLtvList(
    amount: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryLtvList({
      startAfter,
      amount,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryConditionalLtv(
    borrower: string,
    amountToDeposit: number = 0,
    amountToWithdraw: number = 0,
    amountToBorrow: number = 0,
    amountToRepay: number = 0,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryConditionalLtv({
      borrower,
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
      amountToWithdraw: amountToWithdraw
        ? amountToWithdraw.toString()
        : undefined,
      amountToBorrow: amountToBorrow ? amountToBorrow.toString() : undefined,
      amountToRepay: amountToRepay ? amountToRepay.toString() : undefined,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryTotalAvailableToBorrowLiquidity(
    isDisplayed: boolean = false
  ) {
    const res =
      await lendingPlatformQueryClient.queryTotalAvailableToBorrowLiquidity();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryAvailableToBorrow(
    borrower: string,
    targetLtv: number | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryAvailableToBorrow({
      borrower,
      targetLtv: targetLtv ? targetLtv.toString() : undefined,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryAmounts(isDisplayed: boolean = false) {
    const res = await lendingPlatformQueryClient.queryAmounts();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryUserInfo(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await lendingPlatformQueryClient.queryUserInfo({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  // minter

  async function cwMinterQueryConfig(isDisplayed: boolean = false) {
    const res = await minterQueryClient.config();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCurrencyInfo(
    denomOrAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.currencyInfo({ denomOrAddress });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCurrencyInfoList(
    amount: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.currencyInfoList({
      amount,
      startAfter,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCurrencyInfoListByOwner(
    owner: string,
    amount: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.currencyInfoListByOwner({
      owner,
      amount,
      startAfter,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryTokenCountList(
    amount: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.tokenCountList({
      amount,
      startAfter,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwMinterQueryBalances(
    account: string,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.balances({ account });
    return logAndReturn(res, isDisplayed);
  }

  // oracle

  async function cwOracleQueryConfig(isDisplayed: boolean = false) {
    const res = await oracleQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryPrices(
    amount: number = 100,
    startAfter: string | undefined = undefined,
    collections: string[] = [],
    isDisplayed: boolean = false
  ) {
    const msg_1 = {
      amount,
      startAfter,
    };
    const msg_2 = { amount, collections };

    const res = await oracleQueryClient.queryPrices(
      collections.length ? msg_2 : msg_1
    );
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryPrices(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryPricesResponse> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let isOutdated = false;
    let allItems: PriceItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const queryPricesResponse: QueryPricesResponse =
        await oracleQueryClient.queryPrices({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      isOutdated = queryPricesResponse.is_outdated;
      lastItem = getLast(queryPricesResponse.data)?.collection || "";
      allItems = [...allItems, ...queryPricesResponse.data];
      count += queryPricesResponse.data.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(
      {
        is_outdated: isOutdated,
        data: allItems,
      },
      isDisplayed
    );
  }

  async function cwQueryBlockTime(isDisplayed: boolean = false) {
    const res = await oracleQueryClient.queryBlockTime();
    return logAndReturn(res, isDisplayed);
  }

  // market-maker

  async function cwMarketMakerQueryConfig(isDisplayed: boolean = false) {
    const res = await marketMakerQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryOfferPricesList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<OffersResponse[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: OffersResponse[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: OffersResponse[] =
        await marketMakerQueryClient.queryOfferPricesList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.collection_address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryOfferPrices(
    collection: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerQueryClient.queryOfferPrices({
      collection,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryCollectionOwnerList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<CollectionOwnerForAddr[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: CollectionOwnerForAddr[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: CollectionOwnerForAddr[] =
        await marketMakerQueryClient.queryCollectionOwnerList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.collection_address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryCollectionOwner(
    collection: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerQueryClient.queryCollectionOwner({
      collection,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryLiquidityList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<LiquidityInfo[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: LiquidityInfo[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: LiquidityInfo[] =
        await marketMakerQueryClient.queryLiquidityList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.collection_address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryLiquidity(
    collection: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerQueryClient.queryLiquidity({
      collection,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pMarketMakerQueryCollateralList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<CollateralListResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: CollateralListResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: CollateralListResponseItem[] =
        await marketMakerQueryClient.queryCollateralList({
          amount: paginationAmount,
          startAfter: lastItem,
        });

      lastItem = getLast(items)?.collection_owner || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwMarketMakerQueryCollateral(
    collectionOwner: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerQueryClient.queryCollateral({
      collectionOwner,
    });
    return logAndReturn(res, isDisplayed);
  }

  return {
    utils: {
      cwQueryOperators,
      cwQueryApprovals,
      cwQueryBalanceInNft,
      cwQueryNftOwner,
      cwGetMarketMakerData,

      cwV2QueryAsksByCollectionDenom,
      cwV2QueryBidsByTokenPrice,
      cwV2QueryBidsByCreatorCollection,
      cwV2QueryCollectionBidsByPrice,
      cwV2QueryCollectionBid,
    },
    scheduler: {
      cwQueryConfig: cwAdapterSchedulerCommonQueryConfig,
      cwQueryLog: cwAdapterSchedulerCommonQueryLog,
    },
    lending: {
      cwQueryAddressConfig: cwLendingPlatformQueryAddressConfig,
      cwQueryRateConfig: cwLendingPlatformQueryRateConfig,
      cwQueryCommonConfig: cwLendingPlatformQueryCommonConfig,
      cwQueryPlatformRevenue,
      cwQueryBalances,

      cwQueryUnbonder,
      pQueryUnbonderList,

      cwQueryBorrower,
      pQueryBorrowerList,

      cwQueryLiquidator,
      pQueryLiquidatorList,

      cwQueryCollateralByOwner,
      cwQueryCollateral,
      pQueryCollateralList,

      cwQueryLiquidationBidsByCollectionAddress,
      pQueryLiquidationBidsByCollectionAddressList,

      cwQueryLiquidationBidsByLiquidatorAddress,
      cwQueryLiquidationBidsByLiquidatorAddressList,

      cwQueryCollection,
      pQueryCollectionList,

      cwQueryConditionalLtv,
      cwQueryLtvList,

      pQueryProposals,

      cwQueryBglCurrencyToMainCurrencyPrice,
      cwQueryConditionalDepositApr,
      cwQueryTotalAvailableToBorrowLiquidity,
      cwQueryAvailableToBorrow,
      cwQueryAmounts,
      cwQueryUserInfo,
    },
    minter: {
      cwQueryConfig: cwMinterQueryConfig,
      cwQueryCurrencyInfo,
      cwQueryCurrencyInfoList,
      cwQueryCurrencyInfoListByOwner,
      cwQueryTokenCountList,
      cwQueryBalances: cwMinterQueryBalances,
    },
    oracle: {
      cwQueryConfig: cwOracleQueryConfig,

      cwQueryPrices,
      pQueryPrices,

      cwQueryBlockTime,
    },
    marketMaker: {
      cwQueryConfig: cwMarketMakerQueryConfig,

      pQueryOfferPricesList,
      cwQueryOfferPrices,

      pQueryCollectionOwnerList,
      cwQueryCollectionOwner,

      pQueryLiquidityList,
      cwQueryLiquidity,

      pQueryCollateralList: pMarketMakerQueryCollateralList,
      cwQueryCollateral: cwMarketMakerQueryCollateral,
    },
  };
}

export { getCwExecHelpers, getCwQueryHelpers };
