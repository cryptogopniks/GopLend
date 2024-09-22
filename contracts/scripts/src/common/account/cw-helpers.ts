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

import { MarketMakerHubMsgComposer } from "../codegen/MarketMakerHub.message-composer";
import { MarketMakerHubQueryClient } from "../codegen/MarketMakerHub.client";

import { MarketMakerOutpostMsgComposer } from "../codegen/MarketMakerOutpost.message-composer";
import { MarketMakerOutpostQueryClient } from "../codegen/MarketMakerOutpost.client";

import CONFIG_JSON from "../config/config.json";
import { getLast, getPaginationAmount, l, li, logAndReturn } from "../utils";
import { toBase64, fromUtf8 } from "@cosmjs/encoding";
import { getChainOptionById, getContractByLabel } from "../config/config-utils";
import {
  PriceItemForFundsForToken,
  RawPriceItem,
} from "../codegen/Oracle.types";
import { Route } from "../codegen/AdapterDexStargaze.types";
import {
  BidOffset,
  CollectionBidOffset,
  MarketplaceVersion,
  PlatformType,
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
import { OffersResponse } from "../codegen/MarketMakerOutpost.types";
import {
  CollectionInfoForAddr,
  CollectionOwnersForAddr,
  LiquidityInfo,
} from "../codegen/MarketMakerHub.types";
import {
  Bid,
  BidsResponse,
  CollectionBid,
  CollectionBidsResponse,
  CollectionInfoResponse,
  ParamsResponse,
  QueryCollectionInfo,
  QueryParams,
  QueryReverseBidsSortedByPrice,
  QueryReverseCollectionBidsSortedByPrice,
} from "../interfaces/stargaze-marketplace";
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
  MarketplaceData,
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
  let MARKET_MAKER_HUB_CONTRACT: ContractInfo | undefined;
  let MARKET_MAKER_OUTPOST_CONTRACT: ContractInfo | undefined;

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

  try {
    MARKET_MAKER_HUB_CONTRACT = getContractByLabel(
      contracts,
      "market_maker_hub"
    );
  } catch (error) {
    l(error);
  }

  try {
    MARKET_MAKER_OUTPOST_CONTRACT = getContractByLabel(
      contracts,
      "market_maker_outpost"
    );
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
    MARKET_MAKER_HUB_CONTRACT,
    MARKET_MAKER_OUTPOST_CONTRACT,
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
    MARKET_MAKER_HUB_CONTRACT,
    // MARKET_MAKER_OUTPOST_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc, owner, signer);
  if (!cwClient) throw new Error("cwClient is not found!");

  const signingClient = cwClient.client as SigningCosmWasmClient;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

  const adapterDexStargazeMsgComposer = new AdapterDexStargazeMsgComposer(
    owner,
    ADAPTER_DEX_CONTRACT?.ADDRESS || ""
  );

  const adapterMarketplaceStargazeMsgComposer =
    new AdapterMarketplaceStargazeMsgComposer(
      owner,
      ADAPTER_MARKETPLACE_CONTRACT?.ADDRESS || ""
    );

  const adapterSchedulerCommonMsgComposer =
    new AdapterSchedulerCommonMsgComposer(
      owner,
      ADAPTER_SCHEDULER_CONTRACT?.ADDRESS || ""
    );

  const lendingPlatformMainMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_MAIN_CONTRACT?.ADDRESS || ""
  );

  const lendingPlatformStableMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_STABLE_CONTRACT?.ADDRESS || ""
  );

  const minterMsgComposer = new MinterMsgComposer(
    owner,
    MINTER_CONTRACT?.ADDRESS || ""
  );

  const minterKujiraMsgComposer = new MinterKujiraMsgComposer(
    owner,
    MINTER_CONTRACT?.ADDRESS || ""
  );

  const oracleMsgComposer = new OracleMsgComposer(
    owner,
    ORACLE_CONTRACT?.ADDRESS || ""
  );

  const faucetMsgComposer = new FaucetMsgComposer(
    owner,
    FAUCET_CONTRACT?.ADDRESS || ""
  );

  const marketMakerHubMsgComposer = new MarketMakerHubMsgComposer(
    owner,
    MARKET_MAKER_HUB_CONTRACT?.ADDRESS || ""
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
      mainCurrency,
      stableCurrency,
    }: {
      admin?: string;
      worker?: string;
      controller?: string[];
      maxPriceUpdatePeriod?: number;
      executionCooldown?: number;
      mainCurrency?: CurrencyForTokenUnverified;
      stableCurrency?: CurrencyForTokenUnverified;
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
          mainCurrency,
          stableCurrency,
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

  // market-maker-hub

  async function cwMarketMakerHubAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerHubMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwMarketMakerHubUpdateConfig(
    {
      admin,
      worker,
      marketplace,
      controller,
      token,
      maxOutpostAmount,
      outpostCodeId,
    }: {
      admin?: string;
      worker?: string;
      marketplace?: string;
      controller?: string[];
      token?: TokenUnverified;
      maxOutpostAmount?: number;
      outpostCodeId?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerHubMsgComposer.updateConfig({
          admin,
          worker,
          marketplace,
          controller,
          token,
          maxOutpostAmount,
          outpostCodeId,
        }),
      ],
      gasPrice
    );
  }

  async function cwAddOutpost(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerHubMsgComposer.addOutpost()],
      gasPrice
    );
  }

  async function cwMarketMakerHubLock(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerHubMsgComposer.lock()],
      gasPrice
    );
  }

  async function cwMarketMakerHubUnlock(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [marketMakerHubMsgComposer.unlock()],
      gasPrice
    );
  }

  async function cwSetCollection(
    collectionAddress: string,
    ownerList: string[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerHubMsgComposer.setCollection({
          collectionAddress,
          ownerList,
        }),
      ],
      gasPrice
    );
  }

  async function cwRemoveCollections(
    collectionAddressList: string[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [marketMakerHubMsgComposer.removeCollections({ collectionAddressList })],
      gasPrice
    );
  }

  async function cwDepositLiquidity(
    collectionAddress: string,
    amount: number,
    denom: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          marketMakerHubMsgComposer.depositLiquidity({ collectionAddress }),
          amount,
          {
            native: { denom },
          }
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdrawLiquidity(
    collectionAddress: string,
    amount: number,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerHubMsgComposer.withdrawLiquidity({
          collectionAddress,
          amount: amount.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwMarketMakerHubWithdrawCollateral(
    collectionAddress: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [marketMakerHubMsgComposer.withdrawCollateral({ collectionAddress })],
      gasPrice
    );
  }

  async function cwUpdateOffers(
    collectionAddress: string,
    fromToPriceAndOutpostList: [number, number, string][],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        marketMakerHubMsgComposer.updateOffers({
          collectionAddress,
          fromToPriceAndOutpostList: fromToPriceAndOutpostList.map(
            ([priceBefore, priceAfter, outpost]) => [
              priceBefore.toString(),
              priceAfter.toString(),
              outpost,
            ]
          ),
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
        cwBurn,
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
    marketMaker: {
      hub: {
        cwAcceptAdminRole: cwMarketMakerHubAcceptAdminRole,
        cwUpdateConfig: cwMarketMakerHubUpdateConfig,
        cwAddOutpost,
        cwLock: cwMarketMakerHubLock,
        cwUnlock: cwMarketMakerHubUnlock,
        cwSetCollection,
        cwRemoveCollections,
        cwDepositLiquidity,
        cwWithdrawLiquidity,
        cwWithdrawCollateral: cwMarketMakerHubWithdrawCollateral,
        cwUpdateOffers,
      },
      outpost: {},
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
    MARKET_MAKER_HUB_CONTRACT,
    // MARKET_MAKER_OUTPOST_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc);
  if (!cwClient) throw new Error("cwClient is not found!");

  const cosmwasmQueryClient: CosmWasmClient = cwClient.client;

  const adapterDexStargazeQueryClient = new AdapterDexStargazeQueryClient(
    cosmwasmQueryClient,
    ADAPTER_DEX_CONTRACT?.ADDRESS || ""
  );

  const adapterMarketplaceStargazeQueryClient =
    new AdapterMarketplaceStargazeQueryClient(
      cosmwasmQueryClient,
      ADAPTER_MARKETPLACE_CONTRACT?.ADDRESS || ""
    );

  const adapterSchedulerCommonQueryClient =
    new AdapterSchedulerCommonQueryClient(
      cosmwasmQueryClient,
      ADAPTER_SCHEDULER_CONTRACT?.ADDRESS || ""
    );

  const lendingPlatformMainQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_MAIN_CONTRACT?.ADDRESS || ""
  );

  const lendingPlatformStableQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_STABLE_CONTRACT?.ADDRESS || ""
  );

  const minterQueryClient = new MinterQueryClient(
    cosmwasmQueryClient,
    MINTER_CONTRACT?.ADDRESS || ""
  );

  const minterKujiraQueryClient = new MinterKujiraQueryClient(
    cosmwasmQueryClient,
    MINTER_CONTRACT?.ADDRESS || ""
  );

  const oracleQueryClient = new OracleQueryClient(
    cosmwasmQueryClient,
    ORACLE_CONTRACT?.ADDRESS || ""
  );

  const faucetQueryClient = new FaucetQueryClient(
    cosmwasmQueryClient,
    FAUCET_CONTRACT?.ADDRESS || ""
  );

  const marketMakerHubQueryClient = new MarketMakerHubQueryClient(
    cosmwasmQueryClient,
    MARKET_MAKER_HUB_CONTRACT?.ADDRESS || ""
  );

  const getMarketMakerOutpostQueryClient = (outpost: string) => {
    return new MarketMakerOutpostQueryClient(cosmwasmQueryClient, outpost);
  };

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
    const queryMsg: QueryCollectionInfo = {
      collection_info: {},
    };

    const res: CollectionInfoResponse =
      await cosmwasmQueryClient.queryContractSmart(collectionAddress, queryMsg);
    const royaltyFeeRate = Number(res.royalty_info?.share);

    return logAndReturn(royaltyFeeRate, isDisplayed);
  }

  async function pQueryMarketplaceTokenOffers(
    marketplace: string,
    collection: string,
    blockTime: number,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ) {
    const defaultItem: BidOffset = { bidder: "", price: "", token_id: 0 };

    let allItems: Bid[] = [];
    let lastItem: BidOffset | undefined = undefined;
    let count: number = 0;

    while (
      JSON.stringify(lastItem) !== JSON.stringify(defaultItem) &&
      count < (maxCount || count + 1)
    ) {
      const queryMsg: QueryReverseBidsSortedByPrice = {
        reverse_bids_sorted_by_price: {
          collection,
          limit: maxPaginationAmount,
          start_before: lastItem,
        },
      };

      const res: BidsResponse = await cosmwasmQueryClient.queryContractSmart(
        marketplace,
        queryMsg
      );

      const { bidder, price, token_id } = getLast(res.bids) || defaultItem;
      lastItem = { bidder, price, token_id };
      const items = res.bids.filter(
        (x) => Math.floor(Number(x.expires_at) / 1e9) > blockTime
      );
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function pQueryMarketplaceCollectionOffers(
    marketplace: string,
    collection: string,
    blockTime: number,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ) {
    const defaultItem: CollectionBidOffset = {
      bidder: "",
      price: "",
      collection: "",
    };
    let allItems: CollectionBid[] = [];
    let lastItem: CollectionBidOffset | undefined = undefined;
    let count: number = 0;

    while (
      JSON.stringify(lastItem) !== JSON.stringify(defaultItem) &&
      count < (maxCount || count + 1)
    ) {
      const queryMsg: QueryReverseCollectionBidsSortedByPrice = {
        reverse_collection_bids_sorted_by_price: {
          collection,
          limit: maxPaginationAmount,
          start_before: lastItem,
        },
      };

      const res: CollectionBidsResponse =
        await cosmwasmQueryClient.queryContractSmart(marketplace, queryMsg);

      const {
        bidder,
        price,
        collection: lastCollection,
      } = getLast(res.bids) || defaultItem;
      lastItem = { bidder, price, collection: lastCollection };
      const items = res.bids.filter(
        (x) => Math.floor(Number(x.expires_at) / 1e9) > blockTime
      );
      allItems = [...allItems, ...items];
      count += items.length;
    }

    if (maxCount) {
      allItems = allItems.slice(0, maxCount);
    }

    return logAndReturn(allItems, isDisplayed);
  }

  async function cwGetMarketplaceData(
    collectionAndTokensList: [string, string[]][],
    collectionList: string[],
    blockTime: number,
    batchPaginationAmount: number,
    collectionOffersPaginationAmount: number,
    tokenOffersPaginationAmount: number
  ): Promise<MarketplaceData> {
    let promiseList: Promise<void>[] = [];
    let tokenOfferList: [string, Bid[]][] = [];
    let collectionOfferList: [string, CollectionBid[]][] = [];
    let discountBaseList: [string, number][] = [];

    const fn = async (collection: string) => {
      try {
        // query collection offers, fees
        const { collection_bids, discount_base } =
          await cwQueryCollectionOffers(
            collection,
            collectionOffersPaginationAmount
          );

        // remove expired offers
        const marketplaceCollectionOffers = collection_bids.filter(
          (x) => Math.floor(Number(x.expires_at) / 1e9) > blockTime
        ) as CollectionBid[];

        discountBaseList.push([collection, Number(discount_base)]);
        if (marketplaceCollectionOffers.length) {
          collectionOfferList.push([collection, marketplaceCollectionOffers]);
        }

        // query token offers
        const tokenList =
          collectionAndTokensList.find(
            ([collectionAddress]) => collectionAddress === collection
          )?.[1] || [];

        if (tokenList.length) {
          const marketplaceTokenOffers = (await cwQueryTokenOffers(
            collection,
            tokenList,
            tokenOffersPaginationAmount
          )) as [number, Bid[]][];

          // remove expired offers
          const tokenOffers = marketplaceTokenOffers.reduce(
            (acc, [_tokenId, bidList]) => {
              bidList = bidList.filter(
                (x) => Math.floor(Number(x.expires_at) / 1e9) > blockTime
              );
              return [...acc, ...bidList];
            },
            [] as Bid[]
          );

          if (tokenOffers.length) {
            tokenOfferList.push([collection, tokenOffers]);
          }
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

    return {
      tokenOfferList,
      collectionOfferList,
      discountBaseList,
    };
  }

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

  async function cwQueryFloorPrices(
    collectionAddressList: string[],
    isDisplayed: boolean = false
  ) {
    const res = await adapterMarketplaceStargazeQueryClient.queryFloorPrices({
      collectionAddressList,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryTokenOffers(
    collectionAddress: string,
    tokenList: string[],
    limit: number = 100,
    isDisplayed: boolean = false
  ) {
    const res = await adapterMarketplaceStargazeQueryClient.queryTokenOffers({
      collectionAddress,
      tokenList: tokenList.map(Number),
      limit,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCollectionOffers(
    collectionAddress: string,
    limit: number = 100,
    isDisplayed: boolean = false
  ) {
    const res =
      await adapterMarketplaceStargazeQueryClient.queryCollectionOffers({
        collectionAddress,
        limit,
      });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryOffersByCollectionAddress(
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

  async function pQueryUnbonderList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryUnbondersResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryUnbondersResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryUnbondersResponseItem[] =
        await client.queryUnbonderList({
          limit: paginationAmount,
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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryUnbonder({
      address,
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
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryBorrowersResponseItem[] =
        await client.queryBorrowerList({
          limit: paginationAmount,
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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryBorrower({
      address,
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
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryLiquidatorsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryLiquidatorsResponseItem[] =
        await client.queryLiquidatorList({
          limit: paginationAmount,
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
    platformType: PlatformType,
    address: string,
    isDisplayed: boolean = false
  ) {
    const res = await getPlatformQueryClient(platformType).queryLiquidator({
      address,
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
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryCollateralsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryCollateralsResponseItem[] =
        await client.queryCollateralList({
          limit: paginationAmount,
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

  async function pQueryLiquidationBidsByCollectionAddressList(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryLiquidationBidsByCollectionAddressListResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
      [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryLiquidationBidsByCollectionAddressListResponseItem[] =
        await client.queryLiquidationBidsByCollectionAddressList({
          limit: paginationAmount,
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

  async function pQueryProposals(
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryProposalsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryProposalsResponseItem[] = [];
    let firstItem: number | undefined = undefined;
    let count: number = 0;

    while (firstItem !== 0 && count < (maxCount || count + 1)) {
      const items: QueryProposalsResponseItem[] = await client.queryProposals({
        limit: paginationAmount,
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
    platformType: PlatformType,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<QueryCollectionsResponseItem[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);
    const client = getPlatformQueryClient(platformType);

    let allItems: QueryCollectionsResponseItem[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: QueryCollectionsResponseItem[] =
        await client.queryCollectionList({
          limit: paginationAmount,
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
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.currencyInfoList({
      amount: limit,
      startFrom: startAfter,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryCurrencyInfoListByOwner(
    owner: string,
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.currencyInfoListByOwner({
      owner,
      amount: limit,
      startFrom: startAfter,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryTokenCountList(
    limit: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await minterQueryClient.tokenCountList({
      amount: limit,
      startFrom: startAfter,
    });
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
  ): Promise<PriceItemForFundsForToken[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: PriceItemForFundsForToken[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: PriceItemForFundsForToken[] =
        await oracleQueryClient.queryPrices({
          limit: paginationAmount,
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

  // market-maker-hub

  async function cwMarketMakerHubQueryConfig(isDisplayed: boolean = false) {
    const res = await marketMakerHubQueryClient.queryConfig();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryOutposts(isDisplayed: boolean = false) {
    const res = await marketMakerHubQueryClient.queryOutposts();
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryCollectionOwnersList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<CollectionOwnersForAddr[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: CollectionOwnersForAddr[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: CollectionOwnersForAddr[] =
        await marketMakerHubQueryClient.queryCollectionOwnersList({
          limit: paginationAmount,
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

  async function cwQueryCollectionOwners(
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerHubQueryClient.queryCollectionOwners({
      collectionAddress,
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
        await marketMakerHubQueryClient.queryLiquidityList({
          limit: paginationAmount,
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
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerHubQueryClient.queryLiquidity({
      collectionAddress,
    });
    return logAndReturn(res, isDisplayed);
  }

  async function pMarketMakerHubQueryCollateralList(
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<CollectionInfoForAddr[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: CollectionInfoForAddr[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: CollectionInfoForAddr[] =
        await marketMakerHubQueryClient.queryCollateralList({
          limit: paginationAmount,
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

  async function cwMarketMakerHubQueryCollateral(
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await marketMakerHubQueryClient.queryCollateral({
      collectionAddress,
    });
    return logAndReturn(res, isDisplayed);
  }

  // market-maker-outpost

  async function pMarketMakerOutpostQueryCollateralList(
    outpost: string,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<CollectionInfoForAddr[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: CollectionInfoForAddr[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: CollectionInfoForAddr[] =
        await getMarketMakerOutpostQueryClient(outpost).queryCollateralList({
          limit: paginationAmount,
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

  async function cwMarketMakerOutpostQueryCollateral(
    outpost: string,
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await getMarketMakerOutpostQueryClient(outpost).queryCollateral(
      {
        collectionAddress,
      }
    );
    return logAndReturn(res, isDisplayed);
  }

  async function pQueryOfferList(
    outpost: string,
    maxPaginationAmount: number,
    maxCount: number = 0,
    isDisplayed: boolean = false
  ): Promise<OffersResponse[]> {
    const paginationAmount = getPaginationAmount(maxPaginationAmount, maxCount);

    let allItems: OffersResponse[] = [];
    let lastItem: string | undefined = undefined;
    let count: number = 0;

    while (lastItem !== "" && count < (maxCount || count + 1)) {
      const items: OffersResponse[] = await getMarketMakerOutpostQueryClient(
        outpost
      ).queryOfferList({
        limit: paginationAmount,
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

  async function cwQueryOffer(
    outpost: string,
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const res = await getMarketMakerOutpostQueryClient(outpost).queryOffer({
      collectionAddress,
    });
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
      pQueryMarketplaceTokenOffers,
      pQueryMarketplaceCollectionOffers,
      cwGetMarketplaceData,
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
          cwQueryFloorPrices,
          cwQueryTokenOffers,
          cwQueryCollectionOffers,
          cwQueryOffersByCollectionAddress,
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
      kujira: {
        cwQueryCurrenciesByCreator: cwMinterKujiraQueryCurrenciesByCreator,
        cwQueryConfig: cwMinterKujiraQueryConfig,
        cwQueryTokenOwner: cwMinterKujiraQueryTokenOwner,
      },
      stargaze: {
        cwQueryConfig: cwMinterQueryConfig,
        cwQueryCurrencyInfo,
        cwQueryCurrencyInfoList,
        cwQueryCurrencyInfoListByOwner,
        cwQueryTokenCountList,
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
    marketMaker: {
      hub: {
        cwQueryConfig: cwMarketMakerHubQueryConfig,
        cwQueryOutposts,
        pQueryCollectionOwnersList,
        cwQueryCollectionOwners,
        pQueryLiquidityList,
        cwQueryLiquidity,
        pQueryCollateralList: pMarketMakerHubQueryCollateralList,
        cwQueryCollateral: cwMarketMakerHubQueryCollateral,
      },
      outpost: {
        pQueryCollateralList: pMarketMakerOutpostQueryCollateralList,
        cwQueryCollateral: cwMarketMakerOutpostQueryCollateral,
        pQueryOfferList,
        cwQueryOffer,
      },
    },
  };
}

export { getCwExecHelpers, getCwQueryHelpers };
