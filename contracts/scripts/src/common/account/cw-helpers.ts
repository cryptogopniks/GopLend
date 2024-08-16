import { AdapterDexStargazeMsgComposer } from "../codegen/AdapterDexStargaze.message-composer";
import { AdapterDexStargazeQueryClient } from "../codegen/AdapterDexStargaze.client";

import { AdapterMarketplaceStargazeMsgComposer } from "../codegen/AdapterMarketplaceStargaze.message-composer";
import { AdapterMarketplaceStargazeQueryClient } from "../codegen/AdapterMarketplaceStargaze.client";

import { AdapterSchedulerCommonMsgComposer } from "../codegen/AdapterSchedulerCommon.message-composer";
import { AdapterSchedulerCommonQueryClient } from "../codegen/AdapterSchedulerCommon.client";

import { LendingPlatformMsgComposer } from "../codegen/LendingPlatform.message-composer";
import { LendingPlatformQueryClient } from "../codegen/LendingPlatform.client";

import { MinterMsgComposer } from "../codegen/Minter.message-composer";
import { MinterQueryClient } from "../codegen/Minter.client";

import { MinterKujiraMsgComposer } from "../codegen/MinterKujira.message-composer";
import { MinterKujiraQueryClient } from "../codegen/MinterKujira.client";

import { OracleMsgComposer } from "../codegen/Oracle.message-composer";
import { OracleQueryClient } from "../codegen/Oracle.client";

import { FaucetMsgComposer } from "../codegen/Faucet.message-composer";
import { FaucetQueryClient } from "../codegen/Faucet.client";

import CONFIG_JSON from "../config/config.json";
import { getLast, getPaginationAmount, l, li, logAndReturn } from "../utils";
import { toBase64, fromUtf8 } from "@cosmjs/encoding";
import { getChainOptionById, getContractByLabel } from "../config/config-utils";
import { PriceItem, RawPriceItem } from "../codegen/Oracle.types";
import { Route } from "../codegen/AdapterDexStargaze.types";
import {
  BidOffset,
  CollectionBidOffset,
  LiquidationBid,
  MarketplaceVersion,
  PlatformType,
  QueryOffersByCollectionAddressResponse,
} from "../codegen/AdapterMarketplaceStargaze.types";
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
import { ClockSource } from "../codegen/AdapterSchedulerCommon.types";
import {
  InstantiateMarketingInfo,
  Logo,
  Metadata,
} from "../codegen/Minter.types";
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
  Offer,
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

function getContracts(contracts: ContractInfo[]) {
  let ADAPTER_DEX_CONTRACT: ContractInfo | undefined;
  let ADAPTER_MARKETPLACE_CONTRACT: ContractInfo | undefined;
  let ADAPTER_SCHEDULER_CONTRACT: ContractInfo | undefined;
  let LENDING_PLATFORM_MAIN_CONTRACT: ContractInfo | undefined;
  let LENDING_PLATFORM_STABLE_CONTRACT: ContractInfo | undefined;
  let MINTER_CONTRACT: ContractInfo | undefined;
  let ORACLE_CONTRACT: ContractInfo | undefined;
  let FAUCET_CONTRACT: ContractInfo | undefined;

  try {
    ADAPTER_DEX_CONTRACT = getContractByLabel(contracts, "adapter_dex");
  } catch (error) {
    l(error);
  }

  try {
    ADAPTER_MARKETPLACE_CONTRACT = getContractByLabel(
      contracts,
      "adapter_marketplace"
    );
  } catch (error) {
    l(error);
  }

  try {
    ADAPTER_SCHEDULER_CONTRACT = getContractByLabel(
      contracts,
      "adapter_scheduler"
    );
  } catch (error) {
    l(error);
  }

  try {
    LENDING_PLATFORM_MAIN_CONTRACT = getContractByLabel(
      contracts,
      "lending_platform_main"
    );
  } catch (error) {
    l(error);
  }

  try {
    LENDING_PLATFORM_STABLE_CONTRACT = getContractByLabel(
      contracts,
      "lending_platform_stable"
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
    FAUCET_CONTRACT = getContractByLabel(contracts, "faucet");
  } catch (error) {
    l(error);
  }

  return {
    ADAPTER_DEX_CONTRACT,
    ADAPTER_MARKETPLACE_CONTRACT,
    ADAPTER_SCHEDULER_CONTRACT,
    LENDING_PLATFORM_MAIN_CONTRACT,
    LENDING_PLATFORM_STABLE_CONTRACT,
    MINTER_CONTRACT,
    ORACLE_CONTRACT,
    FAUCET_CONTRACT,
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
    ADAPTER_DEX_CONTRACT,
    ADAPTER_MARKETPLACE_CONTRACT,
    ADAPTER_SCHEDULER_CONTRACT,
    LENDING_PLATFORM_MAIN_CONTRACT,
    LENDING_PLATFORM_STABLE_CONTRACT,
    MINTER_CONTRACT,
    ORACLE_CONTRACT,
    FAUCET_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc, owner, signer);
  if (!cwClient) throw new Error("cwClient is not found!");

  const signingClient = cwClient.client as SigningCosmWasmClient;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

  const adapterDexStargazeMsgComposer = new AdapterDexStargazeMsgComposer(
    owner,
    ADAPTER_DEX_CONTRACT ? ADAPTER_DEX_CONTRACT.ADDRESS : ""
  );

  const adapterMarketplaceStargazeMsgComposer =
    new AdapterMarketplaceStargazeMsgComposer(
      owner,
      ADAPTER_MARKETPLACE_CONTRACT ? ADAPTER_MARKETPLACE_CONTRACT.ADDRESS : ""
    );

  const adapterSchedulerCommonMsgComposer =
    new AdapterSchedulerCommonMsgComposer(
      owner,
      ADAPTER_SCHEDULER_CONTRACT ? ADAPTER_SCHEDULER_CONTRACT.ADDRESS : ""
    );

  const lendingPlatformMainMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_MAIN_CONTRACT ? LENDING_PLATFORM_MAIN_CONTRACT.ADDRESS : ""
  );

  const lendingPlatformStableMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_STABLE_CONTRACT
      ? LENDING_PLATFORM_STABLE_CONTRACT.ADDRESS
      : ""
  );

  const minterMsgComposer = new MinterMsgComposer(
    owner,
    MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : ""
  );

  const minterKujiraMsgComposer = new MinterKujiraMsgComposer(
    owner,
    MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : ""
  );

  const oracleMsgComposer = new OracleMsgComposer(
    owner,
    ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : ""
  );

  const faucetMsgComposer = new FaucetMsgComposer(
    owner,
    FAUCET_CONTRACT ? FAUCET_CONTRACT.ADDRESS : ""
  );

  const getPlatformComposer = (platformType: PlatformType) => {
    return platformType === "main"
      ? lendingPlatformMainMsgComposer
      : lendingPlatformStableMsgComposer;
  };

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

  // adapter-dex-stargaze

  async function cwAdapterDexStargazeAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [adapterDexStargazeMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwAdapterDexStargazeUpdateConfig(
    {
      admin,
      worker,
      lendingPlatformStable,
      adapterMarketplace,
      recoverAddress,
      maxPriceImpact,
    }: {
      admin?: string;
      worker?: string;
      lendingPlatformStable?: string;
      adapterMarketplace?: string;
      recoverAddress?: string;
      maxPriceImpact?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterDexStargazeMsgComposer.updateConfig({
          admin,
          worker,
          lendingPlatformStable,
          adapterMarketplace,
          recoverAddress,
          maxPriceImpact: maxPriceImpact?.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwSwap(
    amount: number,
    token: TokenUnverified,
    route: Route,
    priceRatio: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          adapterDexStargazeMsgComposer.swap({
            route,
            priceRatio: priceRatio.toString(),
          }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  // adapter-marketplace-stargaze

  async function cwAdapterMarketplaceStargazeAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [adapterMarketplaceStargazeMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwAdapterMarketplaceStargazeUpdateConfig(
    {
      admin,
      worker,
      lendingPlatformMain,
      lendingPlatformStable,
      oracle,
      marketplace,
      adapterDex,
      marketplaceVersion,
    }: {
      admin?: string;
      worker?: string;
      lendingPlatformMain?: string;
      lendingPlatformStable?: string;
      oracle?: string;
      marketplace?: string;
      adapterDex?: string;
      marketplaceVersion?: MarketplaceVersion;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterMarketplaceStargazeMsgComposer.updateConfig({
          admin,
          worker,
          lendingPlatformMain,
          lendingPlatformStable,
          oracle,
          marketplace,
          adapterDex,
          marketplaceVersion,
        }),
      ],
      gasPrice
    );
  }

  async function cwAcceptOffers(
    route: Route,
    biddedCollateralItemList: BiddedCollateralItem[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterMarketplaceStargazeMsgComposer.acceptOffers({
          biddedCollateralItemList,
          route,
        }),
      ],
      gasPrice
    );
  }

  // adapter-scheduler-common

  async function cwAdapterSchedulerCommonAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [adapterSchedulerCommonMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwAdapterSchedulerCommonUpdateConfig(
    {
      admin,
      worker,
      clockSource,
      lendingPlatformMain,
      lendingPlatformStable,
      executionCooldown,
      offchainClock,
    }: {
      admin?: string;
      worker?: string;
      clockSource?: ClockSource;
      lendingPlatformMain?: string;
      lendingPlatformStable?: string;
      executionCooldown?: number;
      offchainClock?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterSchedulerCommonMsgComposer.updateConfig({
          admin,
          worker,
          lendingPlatformMain,
          lendingPlatformStable,
          offchainClock,
          clockSource,
          executionCooldown,
        }),
      ],
      gasPrice
    );
  }

  async function cwPush(
    route: Route,
    targets: LiquidationItem[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [adapterSchedulerCommonMsgComposer.push({ targets, route })],
      gasPrice
    );
  }

  // lending-platform

  async function cwDeposit(
    platformType: PlatformType,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).deposit(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwUnbond(
    platformType: PlatformType,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).unbond(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdraw(platformType: PlatformType, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).withdraw()],
      gasPrice
    );
  }

  async function cwWithdrawCollateral(
    platformType: PlatformType,
    collections: CollectionInfoForString[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).withdrawCollateral({ collections })],
      gasPrice,
      1.05
    );
  }

  async function cwApproveAndDepositCollateral(
    platformType: PlatformType,
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

    msgList.push(
      getPlatformComposer(platformType).depositCollateral({ collections })
    );

    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }

  async function cwBorrow(
    platformType: PlatformType,
    amount: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).borrow({ amount: `${amount}` })],
      gasPrice
    );
  }

  async function cwRepay(
    platformType: PlatformType,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).repay(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwPlaceBid(
    platformType: PlatformType,
    collections: CollectionInfoForString[],
    discount: number,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).placeBid({
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
    platformType: PlatformType,
    collectionAddresses: string[],
    creationDate: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        getPlatformComposer(platformType).removeBid({
          collectionAddresses,
          creationDate,
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdateBid(
    platformType: PlatformType,
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

    const LENDING_PLATFORM_CONTRACT =
      platformType === "main"
        ? LENDING_PLATFORM_MAIN_CONTRACT
        : LENDING_PLATFORM_STABLE_CONTRACT;

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

    const msgObj = getPlatformComposer(platformType).updateBid({
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

  async function cwLiquidate(
    platformType: PlatformType,
    route: Route,
    targets: LiquidationItem[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).liquidate({ targets, route })],
      gasPrice
    );
  }

  async function cwLendingPlatformAcceptAdminRole(
    platformType: PlatformType,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).acceptAdminRole()],
      gasPrice
    );
  }

  async function cwLendingPlatformUpdateAddressConfig(
    platformType: PlatformType,
    {
      admin,
      worker,
      minter,
      oracle,
      adapterMarketplace,
      adapterScheduler,
    }: {
      admin?: string;
      worker?: string;
      minter?: string;
      oracle?: string;
      adapterMarketplace?: string;
      adapterScheduler?: string;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        getPlatformComposer(platformType).updateAddressConfig({
          admin,
          worker,
          minter,
          oracle,
          adapterMarketplace,
          adapterScheduler,
        }),
      ],
      gasPrice
    );
  }

  async function cwLendingPlatformUpdateRateConfig(
    platformType: PlatformType,
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
        getPlatformComposer(platformType).updateRateConfig({
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
    platformType: PlatformType,
    {
      bglCurrency,
      collateralMinValue,
      isMarketplaceInterfaceEnabled,
      mainCurrency,
      borrowersReserveFractionRatio,
      unbondingPeriod,
    }: {
      bglCurrency?: CurrencyForTokenUnverified;
      collateralMinValue?: number;
      isMarketplaceInterfaceEnabled?: boolean;
      mainCurrency?: CurrencyForTokenUnverified;
      borrowersReserveFractionRatio?: number;
      unbondingPeriod?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        getPlatformComposer(platformType).updateCommonConfig({
          bglCurrency,
          collateralMinValue: collateralMinValue?.toString(),
          isMarketplaceInterfaceEnabled,
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
    platformType: PlatformType,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).depositReserveLiquidity(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdrawReserveLiquidity(
    platformType: PlatformType,
    amount: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        getPlatformComposer(platformType).withdrawReserveLiquidity({
          amount: amount.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwReinforceBglToken(
    platformType: PlatformType,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).reinforceBglToken(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwLock(platformType: PlatformType, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).lock()],
      gasPrice
    );
  }

  async function cwUnlock(platformType: PlatformType, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).unlock()],
      gasPrice
    );
  }

  async function cwDistributeFunds(
    platformType: PlatformType,
    addressAndWeightList: [string, number][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        getPlatformComposer(platformType).distributeFunds({
          addressAndWeightList: addressAndWeightList.map(
            ([address, weight]) => [address, weight.toString()]
          ),
        }),
      ],
      gasPrice
    );
  }

  async function cwRemoveCollection(
    platformType: PlatformType,
    address: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).removeCollection({ address })],
      gasPrice
    );
  }

  async function cwCreateProposal(
    platformType: PlatformType,
    proposal: ProposalForStringAndTokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).createProposal({ proposal })],
      gasPrice
    );
  }

  async function cwRejectProposal(
    platformType: PlatformType,
    id: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getPlatformComposer(platformType).rejectProposal({ id })],
      gasPrice
    );
  }

  async function cwAcceptProposal(
    platformType: PlatformType,
    id: number,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          getPlatformComposer(platformType).acceptProposal({ id }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  // minter

  async function cwCreateNative(
    tokenOwner: string,
    subdenom: string,
    decimals: number,
    paymentAmount: number,
    paymentDenom: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          minterMsgComposer.createNative({ tokenOwner, subdenom, decimals }),
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
    tokenOwner: string,
    name: string,
    symbol: string,
    marketing: InstantiateMarketingInfo,
    decimals: number = 6,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.createCw20({
          tokenOwner,
          name,
          symbol,
          decimals,
          marketing,
        }),
      ],
      gasPrice
    );
  }

  async function cwMint(
    amount: number,
    recipient: string,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.mint({
          token,
          amount: amount.toString(),
          recipient,
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

  async function cwSetMetadataNative(
    token: TokenUnverified,
    metadata: Metadata,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.setMetadataNative({
          token,
          metadata,
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdateMarketingCw20(
    token: TokenUnverified,
    description: string,
    marketing: string,
    project: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateMarketingCw20({
          token,
          project,
          description,
          marketing,
        }),
      ],
      gasPrice
    );
  }

  async function cwUploadLogoCw20(
    token: TokenUnverified,
    logo: Logo,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.uploadLogoCw20({ token, logo })],
      gasPrice
    );
  }

  async function cwChangeAdminNative(token: TokenUnverified, gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.changeAdminNative({ token })],
      gasPrice
    );
  }

  async function cwUpdateMinterCw20(
    token: TokenUnverified,
    newMinter: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateMinterCw20({
          token,
          newMinter,
        }),
      ],
      gasPrice
    );
  }

  async function cwRegisterCurrency(
    currency: CurrencyForTokenUnverified,
    creator: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.registerCurrency({
          currency,
          creator,
        }),
      ],
      gasPrice
    );
  }

  async function cwMinterAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwMinterUpdateConfig(
    {
      admin,
      worker,
      cw20BaseCodeId,
    }: {
      admin?: string;
      worker?: string;
      cw20BaseCodeId?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterMsgComposer.updateConfig({
          admin,
          worker,
          cw20BaseCodeId,
        }),
      ],
      gasPrice
    );
  }

  async function cwMinterUpdateTokenOwner(
    denom: string,
    owner: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [minterMsgComposer.updateTokenOwner({ denom, owner })],
      gasPrice
    );
  }

  // minter-kujira

  async function cwMinterKujiraCreateNative(
    tokenOwner: string,
    subdenom: string,
    decimals: number,
    paymentAmount: number,
    paymentDenom: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          minterKujiraMsgComposer.createNative({
            tokenOwner,
            subdenom,
            decimals,
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

  async function cwMinterKujiraMint(
    amount: number,
    recipient: string,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterKujiraMsgComposer.mint({
          token,
          amount: amount.toString(),
          recipient,
        }),
      ],
      gasPrice
    );
  }

  async function cwMinterKujiraBurn(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          minterKujiraMsgComposer.burn(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwMinterKujiraRegisterCurrency(
    currency: CurrencyForTokenUnverified,
    creator: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterKujiraMsgComposer.registerCurrency({
          currency,
          creator,
        }),
      ],
      gasPrice
    );
  }

  async function cwMinterKujiraAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [minterKujiraMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwMinterKujiraUpdateConfig(
    {
      admin,
      worker,
    }: {
      admin?: string;
      worker?: string;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        minterKujiraMsgComposer.updateConfig({
          admin,
          worker,
        }),
      ],
      gasPrice
    );
  }

  async function cwMinterKujiraUpdateTokenOwner(
    denom: string,
    owner: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [minterKujiraMsgComposer.updateTokenOwner({ denom, owner })],
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

  async function cwSetPrices(
    priceRatio: number,
    data: RawPriceItem[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        oracleMsgComposer.setPrices({
          data,
          priceRatio: priceRatio.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwUpdatePrices(
    priceRatio: number,
    data: RawPriceItem[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        oracleMsgComposer.updatePrices({
          data,
          priceRatio: priceRatio.toString(),
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

  // faucet

  async function cwClaim(gasPrice: string) {
    return await _msgWrapperWithGasPrice([faucetMsgComposer.claim()], gasPrice);
  }

  async function cwClaimFor(
    addressAndAmountList: [string, number][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        faucetMsgComposer.claimFor({
          addressAndAmountList: addressAndAmountList.map(
            ([address, amount]) => [address, amount.toString()]
          ),
        }),
      ],
      gasPrice
    );
  }

  async function cwFaucetUpdateConfig(
    {
      claimableAmount,
      claimCooldown,
    }: {
      claimableAmount?: number;
      claimCooldown?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        faucetMsgComposer.updateConfig({
          claimableAmount: claimableAmount?.toString(),
          claimCooldown,
        }),
      ],
      gasPrice
    );
  }

  return {
    utils: { cwRevoke },
    adapter: {
      dex: {
        kujira: {},
        stargaze: {
          cwAcceptAdminRole: cwAdapterDexStargazeAcceptAdminRole,
          cwUpdateConfig: cwAdapterDexStargazeUpdateConfig,
          cwSwap,
        },
      },
      marketplace: {
        kujira: {},
        stargaze: {
          cwAcceptAdminRole: cwAdapterMarketplaceStargazeAcceptAdminRole,
          cwUpdateConfig: cwAdapterMarketplaceStargazeUpdateConfig,
          cwAcceptOffers,
        },
      },
      scheduler: {
        kujira: {
          cwAcceptAdminRole: cwAdapterSchedulerCommonAcceptAdminRole,
          cwUpdateConfig: cwAdapterSchedulerCommonUpdateConfig,
          cwPush,
        },
        stargaze: {
          cwAcceptAdminRole: cwAdapterSchedulerCommonAcceptAdminRole,
          cwUpdateConfig: cwAdapterSchedulerCommonUpdateConfig,
          cwPush,
        },
      },
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
      cwLock,
      cwUnlock,
      cwDistributeFunds,
      cwRemoveCollection,
      cwCreateProposal,
      cwRejectProposal,
      cwAcceptProposal,
    },
    minter: {
      kujira: {
        cwCreateNative: cwMinterKujiraCreateNative,
        cwMint: cwMinterKujiraMint,
        cwBurn: cwMinterKujiraBurn,
        cwRegisterCurrency: cwMinterKujiraRegisterCurrency,
        cwAcceptAdminRole: cwMinterKujiraAcceptAdminRole,
        cwUpdateConfig: cwMinterKujiraUpdateConfig,
        cwUpdateTokenOwner: cwMinterKujiraUpdateTokenOwner,
      },
      stargaze: {
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
        cwAcceptAdminRole: cwMinterAcceptAdminRole,
        cwUpdateConfig: cwMinterUpdateConfig,
        cwUpdateTokenOwner: cwMinterUpdateTokenOwner,
      },
    },
    oracle: {
      cwAcceptAdminRole: cwOracleAcceptAdminRole,
      cwUpdateConfig: cwOracleUpdateConfig,
      cwSetPrices,
      cwUpdatePrices,
      cwRemovePrices,
    },
    faucet: {
      kujira: {},
      stargaze: {
        cwClaim,
        cwClaimFor,
        cwUpdateConfig: cwFaucetUpdateConfig,
      },
    },
  };
}

async function getCwQueryHelpers(chainId: string, rpc: string) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const {
    ADAPTER_DEX_CONTRACT,
    ADAPTER_MARKETPLACE_CONTRACT,
    ADAPTER_SCHEDULER_CONTRACT,
    LENDING_PLATFORM_MAIN_CONTRACT,
    LENDING_PLATFORM_STABLE_CONTRACT,
    MINTER_CONTRACT,
    ORACLE_CONTRACT,
    FAUCET_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc);
  if (!cwClient) throw new Error("cwClient is not found!");

  const cosmwasmQueryClient: CosmWasmClient = cwClient.client;

  const adapterDexStargazeQueryClient = new AdapterDexStargazeQueryClient(
    cosmwasmQueryClient,
    ADAPTER_DEX_CONTRACT ? ADAPTER_DEX_CONTRACT.ADDRESS : ""
  );

  const adapterMarketplaceStargazeQueryClient =
    new AdapterMarketplaceStargazeQueryClient(
      cosmwasmQueryClient,
      ADAPTER_MARKETPLACE_CONTRACT ? ADAPTER_MARKETPLACE_CONTRACT.ADDRESS : ""
    );

  const adapterSchedulerCommonQueryClient =
    new AdapterSchedulerCommonQueryClient(
      cosmwasmQueryClient,
      ADAPTER_SCHEDULER_CONTRACT ? ADAPTER_SCHEDULER_CONTRACT.ADDRESS : ""
    );

  const lendingPlatformMainQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_MAIN_CONTRACT ? LENDING_PLATFORM_MAIN_CONTRACT.ADDRESS : ""
  );

  const lendingPlatformStableQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_STABLE_CONTRACT
      ? LENDING_PLATFORM_STABLE_CONTRACT.ADDRESS
      : ""
  );

  const minterQueryClient = new MinterQueryClient(
    cosmwasmQueryClient,
    MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : ""
  );

  const minterKujiraQueryClient = new MinterKujiraQueryClient(
    cosmwasmQueryClient,
    MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : ""
  );

  const oracleQueryClient = new OracleQueryClient(
    cosmwasmQueryClient,
    ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : ""
  );

  const faucetQueryClient = new FaucetQueryClient(
    cosmwasmQueryClient,
    FAUCET_CONTRACT ? FAUCET_CONTRACT.ADDRESS : ""
  );

  const getPlatformQueryClient = (platformType: PlatformType) => {
    return platformType === "main"
      ? lendingPlatformMainQueryClient
      : lendingPlatformStableQueryClient;
  };

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

  async function cwQueryNetworkFee(
    stargazeMarketplace: string,
    isDisplayed: boolean = false
  ) {
    type Duration = { height: number } | { time: number };

    interface ExpiryRange {
      min: number;
      max: number;
    }

    interface QueryParams {
      params: {};
    }

    interface ParamsResponse {
      params: {
        /// Fair Burn fee for winning bids
        trading_fee_percent: String;
        /// Valid time range for Asks
        /// (min, max) in seconds
        ask_expiry: ExpiryRange;
        /// Valid time range for Bids
        /// (min, max) in seconds
        bid_expiry: ExpiryRange;
        /// Operators are entites that are responsible for maintaining the active state of Asks
        /// They listen to NFT transfer events, and update the active state of Asks
        operators: string[];
        /// Max value for the finders fee
        max_finders_fee_percent: string;
        /// Min value for a bid
        min_price: string;
        /// Duration after expiry when a bid becomes stale
        stale_bid_duration: Duration;
        /// Stale bid removal reward
        bid_removal_reward_percent: string;
        /// Listing fee to reduce spam
        listing_fee: string;
      };
    }

    const queryMsg: QueryParams = {
      params: {},
    };

    const res: ParamsResponse = await cosmwasmQueryClient.queryContractSmart(
      stargazeMarketplace,
      queryMsg
    );

    const networkFeeRate = Number(res.params.trading_fee_percent) / 100;

    return logAndReturn(networkFeeRate, isDisplayed);
  }

  async function cwQueryRoyaltyFee(
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    interface QueryCollectionInfo {
      collection_info: {};
    }

    const queryMsg: QueryCollectionInfo = {
      collection_info: {},
    };

    interface RoyaltyInfoResponse {
      payment_address: string;
      share: string;
    }

    interface Timestamp {
      timestamp: number;
    }

    interface CollectionInfoResponse {
      creator: String;
      description: String;
      image: String;
      external_link?: string;
      explicit_content?: boolean;
      start_trading_time?: Timestamp;
      royalty_info?: RoyaltyInfoResponse;
    }

    const res: CollectionInfoResponse =
      await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryMsg);

    const royaltyFeeRate = Number(res.royalty_info?.share || "0") / 100;

    return logAndReturn(royaltyFeeRate, isDisplayed);
  }

  async function cwQueryFinderFee(
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    interface QueryCollectionInfo {
      collection_info: {};
    }

    const queryMsg: QueryCollectionInfo = {
      collection_info: {},
    };

    interface RoyaltyInfoResponse {
      payment_address: string;
      share: string;
    }

    interface Timestamp {
      timestamp: number;
    }

    interface CollectionInfoResponse {
      creator: String;
      description: String;
      image: String;
      external_link?: string;
      explicit_content?: boolean;
      start_trading_time?: Timestamp;
      royalty_info?: RoyaltyInfoResponse;
    }

    const res: CollectionInfoResponse =
      await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryMsg);

    const royaltyFeeRate = Number(res.royalty_info?.share || "0") / 100;

    return logAndReturn(royaltyFeeRate, isDisplayed);
  }

  // async function cwQueryMarketplaceBids(
  //   marketplace: string,
  //   collection: string,
  //   isDisplayed: boolean = false
  // ) {
  //   interface BidOffset {
  //     price: string;
  //     token_id: number;
  //     bidder: string;
  //   }

  //   interface QueryReverseBidsSortedByPrice {
  //     reverse_bids_sorted_by_price: {
  //       collection: string;
  //       start_before?: BidOffset;
  //       limit?: number;
  //     };
  //   }

  //   interface Timestamp {
  //     timestamp: string;
  //   }

  //   interface Bid {
  //     collection: string;
  //     token_id: number;
  //     bidder: string;
  //     price: string;
  //     finders_fee_bps?: number;
  //     expires_at: Timestamp;
  //   }

  //   interface BidsResponse {
  //     bids: Bid[];
  //   }

  //   const limit: number = 100;
  //   const defaultItem: BidOffset = { bidder: "", price: "", token_id: 0 };
  //   let allItems: Bid[] = [];
  //   let lastItem: BidOffset | undefined = undefined;

  //   while (JSON.stringify(lastItem) !== JSON.stringify(defaultItem)) {
  //     const queryMsg: QueryReverseBidsSortedByPrice = {
  //       reverse_bids_sorted_by_price: {
  //         collection,
  //         limit,
  //         start_before: lastItem,
  //       },
  //     };

  //     const res: BidsResponse = await cosmwasmQueryClient.queryContractSmart(
  //       marketplace,
  //       queryMsg
  //     );

  //     const { bidder, price, token_id } = getLast(res.bids) || defaultItem;
  //     lastItem = { bidder, price, token_id };
  //     allItems = [...allItems, ...res.bids];

  //     l({ lastOffer: lastItem });
  //   }

  //   return logAndReturn(allItems, isDisplayed);
  // }

  // async function cwQueryMarketplaceCollectionBids(
  //   marketplace: string,
  //   collection: string,
  //   isDisplayed: boolean = false
  // ) {
  //   interface CollectionBidOffset {
  //     price: string;
  //     collection: string;
  //     bidder: string;
  //   }

  //   interface QueryReverseCollectionBidsSortedByPrice {
  //     reverse_collection_bids_sorted_by_price: {
  //       collection: string;
  //       start_before?: CollectionBidOffset;
  //       limit?: number;
  //     };
  //   }

  //   interface CollectionBid {
  //     collection: string;
  //     bidder: string;
  //     price: string;
  //     finders_fee_bps?: number;
  //     expires_at: string;
  //   }

  //   interface CollectionBidsResponse {
  //     bids: CollectionBid[];
  //   }

  //   const limit: number = 100;
  //   const defaultItem: CollectionBidOffset = {
  //     bidder: "",
  //     price: "",
  //     collection: "",
  //   };
  //   let allItems: CollectionBid[] = [];
  //   let lastItem: CollectionBidOffset | undefined = undefined;

  //   while (JSON.stringify(lastItem) !== JSON.stringify(defaultItem)) {
  //     const queryMsg: QueryReverseCollectionBidsSortedByPrice = {
  //       reverse_collection_bids_sorted_by_price: {
  //         collection,
  //         limit,
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
  //     allItems = [...allItems, ...res.bids];

  //     l({ lastOffer: lastItem });
  //   }

  //   return logAndReturn(allItems, isDisplayed);
  // }

  // query first 10 token offers and 10 collection offers
  async function cwQueryMarketplaceOffers(
    marketplace: string,
    collection: string,
    limit: number = 10, // the value will be limited by 10 in marketplace contract
    isDisplayed: boolean = false
  ) {
    const getBids = async (): Promise<Offer[]> => {
      interface BidOffset {
        price: string;
        token_id: number;
        bidder: string;
      }

      interface QueryReverseBidsSortedByPrice {
        reverse_bids_sorted_by_price: {
          collection: string;
          start_before?: BidOffset;
          limit?: number;
        };
      }

      interface Bid {
        collection: string;
        token_id: number;
        bidder: string;
        price: string;
        finders_fee_bps?: number;
        expires_at: string;
      }

      interface BidsResponse {
        bids: Bid[];
      }

      const queryMsg: QueryReverseBidsSortedByPrice = {
        reverse_bids_sorted_by_price: {
          collection,
          limit,
        },
      };

      const res: BidsResponse = await cosmwasmQueryClient.queryContractSmart(
        marketplace,
        queryMsg
      );

      return res.bids.map((x) => ({
        price: Number(x.price),
        expires_at: Math.floor(Number(x.expires_at) / 1e9),
      }));
    };

    const getCollectionBids = async (): Promise<Offer[]> => {
      interface CollectionBidOffset {
        price: string;
        collection: string;
        bidder: string;
      }

      interface QueryReverseCollectionBidsSortedByPrice {
        reverse_collection_bids_sorted_by_price: {
          collection: string;
          start_before?: CollectionBidOffset;
          limit?: number;
        };
      }

      interface CollectionBid {
        collection: string;
        bidder: string;
        price: string;
        finders_fee_bps?: number;
        expires_at: string;
      }

      interface CollectionBidsResponse {
        bids: CollectionBid[];
      }

      const queryMsg: QueryReverseCollectionBidsSortedByPrice = {
        reverse_collection_bids_sorted_by_price: {
          collection,
          limit,
        },
      };

      const res: CollectionBidsResponse =
        await cosmwasmQueryClient.queryContractSmart(marketplace, queryMsg);

      return res.bids.map((x) => ({
        price: Number(x.price),
        expires_at: Math.floor(Number(x.expires_at) / 1e9),
      }));
    };

    const [bids, collectionBids] = await Promise.all([
      getBids(),
      getCollectionBids(),
    ]);
    const offers = [...bids, ...collectionBids];

    return logAndReturn(offers, isDisplayed);
  }

  // query ALL token and collection offers
  async function cwGetMarketplaceOffers(
    platformType: PlatformType,
    collectionAddressList: string[],
    is_marketplace_interface_enabled: boolean,
    isDisplayed: boolean = false
  ): Promise<QueryLiquidationBidsByCollectionAddressListResponseItem[]> {
    const EXPIRATION_DELAY_S = 30;
    const ASYNC_QUERY_AMOUNT = 5;

    if (!is_marketplace_interface_enabled) {
      return [];
    }

    const tokenOfferDefault: BidOffset = { bidder: "", price: "", token_id: 0 };
    const collectionOfferDefault: CollectionBidOffset = {
      bidder: "",
      price: "",
      collection: "",
    };

    const tokenOfferDefaultString = JSON.stringify(tokenOfferDefault);
    const collectionOfferDefaultString = JSON.stringify(collectionOfferDefault);

    let promiseList: Promise<void>[] = [];
    let allItems: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
      [];

    for (const collectionAddress of collectionAddressList) {
      const fn = async () => {
        try {
          let offers: LiquidationBid[] = [];
          let lastTokenOfferItem: BidOffset | undefined = undefined;
          let lastCollectionOfferItem: CollectionBidOffset | undefined =
            undefined;

          while (
            !(
              JSON.stringify(lastTokenOfferItem) === tokenOfferDefaultString &&
              JSON.stringify(lastCollectionOfferItem) ===
                collectionOfferDefaultString
            )
          ) {
            const items: QueryOffersByCollectionAddressResponse =
              await cwAdapterMarketplaceStargazeQueryOffersByCollectionAddress(
                platformType,
                collectionAddress,
                EXPIRATION_DELAY_S,
                lastCollectionOfferItem,
                lastTokenOfferItem
              );

            lastTokenOfferItem = items.last_token_offer || tokenOfferDefault;
            lastCollectionOfferItem =
              items.last_collection_offer || collectionOfferDefault;

            offers = [...offers, ...items.offers];
          }

          allItems.push({
            collection_address: collectionAddress,
            liquidation_bids: offers,
          });
        } catch (_) {}
      };

      if (promiseList.length >= ASYNC_QUERY_AMOUNT) {
        await Promise.all(promiseList);
        promiseList = [];
      }

      promiseList.push(fn());
    }

    return logAndReturn(allItems, isDisplayed);
  }

  // async function getLtvList(platformType: PlatformType) {
  //   const PAGINATION_QUERY_AMOUNT = 200;

  //   let allItems: [Addr, Decimal | null][] = [];
  //   let lastItem: string | undefined = undefined;

  //   while (lastItem !== "") {
  //     const items: [Addr, Decimal | null][] = await cwQueryLtvList(
  //       platformType,
  //       PAGINATION_QUERY_AMOUNT,
  //       lastItem
  //     );

  //     lastItem = getLast(items)?.[0] || "";
  //     allItems = [...allItems, ...items];

  //     l({ lastLtv: lastItem });
  //   }

  //   return allItems;
  // }

  // adapter-dex-stargaze

  async function cwAdapterDexStargazeQueryConfig(isDisplayed: boolean = false) {
    const res = await adapterDexStargazeQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  // adapter-marketplace-stargaze

  async function cwAdapterMarketplaceStargazeQueryConfig(
    isDisplayed: boolean = false
  ) {
    const res = await adapterMarketplaceStargazeQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwAdapterMarketplaceStargazeQueryOffersByCollectionAddress(
    platformType: PlatformType,
    collectionAddress: string,
    expirationDelay: number,
    startAfterCollectionOffer?: CollectionBidOffset,
    startAfterTokenOffer?: BidOffset,
    limit: number = 10,
    isDisplayed: boolean = false
  ) {
    const res =
      await adapterMarketplaceStargazeQueryClient.queryOffersByCollectionAddress(
        {
          platformType,
          collectionAddress,
          expirationDelay,
          startAfterCollectionOffer,
          startAfterTokenOffer,
          limit,
        }
      );
    return logAndReturn(res, isDisplayed);
  }

  // adapter-scheduler-common

  async function cwAdapterSchedulerCommonQueryConfig(
    isDisplayed: boolean = false
  ) {
    const res = await adapterSchedulerCommonQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwAdapterSchedulerCommonQueryLog(
    isDisplayed: boolean = false
  ) {
    const res = await adapterSchedulerCommonQueryClient.queryLog();
    return logAndReturn(res, isDisplayed);
  }

  // lending-platform

  async function cwLendingPlatformQueryAddressConfig(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryAddressConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwLendingPlatformQueryRateConfig(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryRateConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwLendingPlatformQueryCommonConfig(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryCommonConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryPlatformRevenue(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryPlatformRevenue();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBalances(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryBalances();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryUnbonderList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryUnbonderList({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryUnbonderList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryUnbondersResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryUnbondersResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryUnbondersResponseItem[] = await cwQueryUnbonderList(
        platformType,
        paginationAmount,
        lastItem
      );

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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryUnbonder({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBorrowerList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryBorrowerList({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryBorrowerList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryBorrowersResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryBorrowersResponseItem[] = await cwQueryBorrowerList(
        platformType,
        paginationAmount,
        lastItem
      );

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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryBorrower({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLiquidatorList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryLiquidatorList({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryLiquidatorList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryLiquidatorsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryLiquidatorsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryLiquidatorsResponseItem[] = await cwQueryLiquidatorList(
        platformType,
        paginationAmount,
        lastItem
      );

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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryLiquidator({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCollateralList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryCollateralList({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryCollateralList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryCollateralsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryCollateralsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryCollateralsResponseItem[] = await cwQueryCollateralList(
        platformType,
        paginationAmount,
        lastItem
      );

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
    platformType: PlatformType,
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryCollateral({
      collectionAddress,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCollateralByOwner(
    platformType: PlatformType,
    owner: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryCollateralByOwner({
      owner,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLiquidationBidsByCollectionAddressList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryLiquidationBidsByCollectionAddressList({ startAfter, limit });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryLiquidationBidsByCollectionAddressList(
    platformType: PlatformType,
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
        await cwQueryLiquidationBidsByCollectionAddressList(
          platformType,
          paginationAmount,
          lastItem
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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryLiquidationBidsByCollectionAddress({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLiquidationBidsByLiquidatorAddressList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryLiquidationBidsByLiquidatorAddressList({ startAfter, limit });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLiquidationBidsByLiquidatorAddress(
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryLiquidationBidsByLiquidatorAddress({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryProposals(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: number | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryProposals({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryProposals(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryProposalsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryProposalsResponseItem[] = [];
    let firstItem: number | undefined = undefined;
    let count: number = 0;

    while (firstItem !== 0 && count < (maxCount || count + 1)) {
      const items: QueryProposalsResponseItem[] = await cwQueryProposals(
        platformType,
        paginationAmount,
        firstItem
      );

      firstItem = getLast(items)?.id || 0;
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryCollectionList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryCollectionList({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryCollectionList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryCollectionsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: QueryCollectionsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryCollectionsResponseItem[] = await cwQueryCollectionList(
        platformType,
        paginationAmount,
        lastItem
      );

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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryCollection({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBglCurrencyToMainCurrencyPrice(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryBglCurrencyToMainCurrencyPrice();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryConditionalDepositApr(
    platformType: PlatformType,
    amountToDeposit: number,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryConditionalDepositApr({
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLtvList(
    platformType: PlatformType,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryLtvList({
      startAfter,
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryConditionalLtv(
    platformType: PlatformType,
    borrower: string,
    amountToDeposit: number = 0,
    amountToBorrow: number = 0,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryConditionalLtv({
      borrower,
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
      amountToBorrow: amountToBorrow ? amountToBorrow.toString() : undefined,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryTotalAvailableToBorrowLiquidity(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryTotalAvailableToBorrowLiquidity();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryAvailableToBorrow(
    platformType: PlatformType,
    borrower: string,
    targetLtv: number | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(
      platformType
    ).queryAvailableToBorrow({
      borrower,
      targetLtv: targetLtv ? targetLtv.toString() : undefined,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryAmounts(
    platformType: PlatformType,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryAmounts();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryUserInfo(
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryUserInfo({
      address,
    });
    return logAndReturn(res, isDisplayed);
  }

  // minter

  async function cwQueryCurrenciesByCreator(
    creator: string,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.queryCurrenciesByCreator({ creator });
    return logAndReturn(res, isDisplayed);
  }

  async function cwMinterQueryConfig(isDisplayed: boolean = false) {
    const res = await minterQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwMinterQueryTokenOwner(
    denom: string,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.queryTokenOwner({ denom });
    return logAndReturn(res, isDisplayed);
  }

  // minter-kujira

  async function cwMinterKujiraQueryCurrenciesByCreator(
    creator: string,
    isDisplayed: boolean = false
  ) {
    const res = await minterKujiraQueryClient.queryCurrenciesByCreator({
      creator,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwMinterKujiraQueryConfig(isDisplayed: boolean = false) {
    const res = await minterKujiraQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwMinterKujiraQueryTokenOwner(
    denom: string,
    isDisplayed: boolean = false
  ) {
    const res = await minterKujiraQueryClient.queryTokenOwner({ denom });
    return logAndReturn(res, isDisplayed);
  }

  // oracle

  async function cwOracleQueryConfig(isDisplayed: boolean = false) {
    const res = await oracleQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryPrices(
    limit: number = 100,
    startAfter: string | undefined = undefined,
    collectionAddresses: string[] = [],
    isDisplayed: boolean = false
  ) {
    const msg_1 = {
      startAfter,
      limit,
    };
    const msg_2 = { collectionAddresses };

    const res = await oracleQueryClient.queryPrices(
      collectionAddresses.length ? msg_2 : msg_1
    );
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryPrices(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<PriceItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: PriceItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: PriceItem[] = await cwQueryPrices(
        paginationAmount,
        lastItem
      );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwQueryPriceRatio(isDisplayed: boolean = false) {
    const res = await oracleQueryClient.queryPriceRatio();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBlockTime(isDisplayed: boolean = false) {
    const res = await oracleQueryClient.queryBlockTime();
    return logAndReturn(res, isDisplayed);
  }

  async function cwOracleQueryLog(isDisplayed: boolean = false) {
    const res = await oracleQueryClient.queryLog();
    return logAndReturn(res, isDisplayed);
  }

  // faucet

  async function cwQueryDenom(isDisplayed: boolean = false) {
    const res = await faucetQueryClient.queryDenom();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryLastClaimDate(
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await faucetQueryClient.queryLastClaimDate({ address });
    return logAndReturn(res, isDisplayed);
  }

  async function cwFaucetQueryConfig(isDisplayed: boolean = false) {
    const res = await faucetQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  return {
    utils: {
      cwQueryOperators,
      cwQueryApprovals,
      cwQueryBalanceInNft,
      cwQueryNftOwner,
      cwQueryNetworkFee,
      cwQueryRoyaltyFee,
      cwQueryFinderFee,
      // cwQueryMarketplaceBids,
      // cwQueryMarketplaceCollectionBids,
      cwQueryMarketplaceOffers,
      cwGetMarketplaceOffers,
    },
    adapter: {
      dex: {
        kujira: {},
        stargaze: {
          cwQueryConfig: cwAdapterDexStargazeQueryConfig,
        },
      },
      marketplace: {
        kujira: {},
        stargaze: {
          cwQueryConfig: cwAdapterMarketplaceStargazeQueryConfig,
          cwQueryOffersByCollectionAddress:
            cwAdapterMarketplaceStargazeQueryOffersByCollectionAddress,
        },
      },
      scheduler: {
        kujira: {
          cwQueryConfig: cwAdapterSchedulerCommonQueryConfig,
          cwQueryLog: cwAdapterSchedulerCommonQueryLog,
        },
        stargaze: {
          cwQueryConfig: cwAdapterSchedulerCommonQueryConfig,
          cwQueryLog: cwAdapterSchedulerCommonQueryLog,
        },
      },
    },
    lending: {
      cwQueryAddressConfig: cwLendingPlatformQueryAddressConfig,
      cwQueryRateConfig: cwLendingPlatformQueryRateConfig,
      cwQueryCommonConfig: cwLendingPlatformQueryCommonConfig,
      cwQueryPlatformRevenue,
      cwQueryBalances,

      cwQueryUnbonder,
      cwQueryUnbonderList,
      pQueryUnbonderList,

      cwQueryBorrower,
      cwQueryBorrowerList,
      pQueryBorrowerList,

      cwQueryLiquidator,
      cwQueryLiquidatorList,
      pQueryLiquidatorList,

      cwQueryCollateralByOwner,
      cwQueryCollateral,
      cwQueryCollateralList,
      pQueryCollateralList,

      cwQueryLiquidationBidsByCollectionAddress,
      cwQueryLiquidationBidsByCollectionAddressList,
      pQueryLiquidationBidsByCollectionAddressList,

      cwQueryLiquidationBidsByLiquidatorAddress,
      cwQueryLiquidationBidsByLiquidatorAddressList,

      cwQueryCollection,
      cwQueryCollectionList,
      pQueryCollectionList,

      cwQueryConditionalLtv,
      cwQueryLtvList,

      cwQueryProposals,
      pQueryProposals,

      cwQueryBglCurrencyToMainCurrencyPrice,
      cwQueryConditionalDepositApr,
      cwQueryTotalAvailableToBorrowLiquidity,
      cwQueryAvailableToBorrow,
      cwQueryAmounts,
      cwQueryUserInfo,
    },
    minter: {
      kujira: {
        cwQueryCurrenciesByCreator: cwMinterKujiraQueryCurrenciesByCreator,
        cwQueryConfig: cwMinterKujiraQueryConfig,
        cwQueryTokenOwner: cwMinterKujiraQueryTokenOwner,
      },
      stargaze: {
        cwQueryCurrenciesByCreator,
        cwQueryConfig: cwMinterQueryConfig,
        cwQueryTokenOwner: cwMinterQueryTokenOwner,
      },
    },
    oracle: {
      cwQueryConfig: cwOracleQueryConfig,

      cwQueryPrices,
      pQueryPrices,

      cwQueryPriceRatio,
      cwQueryBlockTime,
      cwQueryLog: cwOracleQueryLog,
    },
    faucet: {
      kujira: {},
      stargaze: {
        cwQueryDenom,
        cwQueryLastClaimDate,
        cwQueryConfig: cwFaucetQueryConfig,
      },
    },
  };
}

export { getCwExecHelpers, getCwQueryHelpers };
