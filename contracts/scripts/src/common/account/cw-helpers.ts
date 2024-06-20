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

import CONFIG_JSON from "../config/config.json";
import { getLast, l, logAndReturn } from "../utils";
import { toBase64, fromUtf8 } from "@cosmjs/encoding";
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
  ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
  BiddedCollateralItem,
  CollectionInfoForString,
  CurrencyForTokenUnverified,
  LiquidationItem,
  ProposalForStringAndTokenUnverified,
  QueryMsg as LendingPlatformQueryMsg,
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
} from "../interfaces";
import { Route } from "../codegen/AdapterDexStargaze.types";

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
  let ADAPTER_DEX_STARGAZE_CONTRACT: ContractInfo | undefined;
  let ADAPTER_MARKETPLACE_STARGAZE_CONTRACT: ContractInfo | undefined;
  let ADAPTER_SCHEDULER_COMMON_CONTRACT: ContractInfo | undefined;
  let LENDING_PLATFORM_CONTRACT: ContractInfo | undefined;
  let MINTER_CONTRACT: ContractInfo | undefined;
  let MINTER_KUJIRA_CONTRACT: ContractInfo | undefined;
  let ORACLE_CONTRACT: ContractInfo | undefined;

  try {
    ADAPTER_DEX_STARGAZE_CONTRACT = getContractByWasm(
      contracts,
      "adapter_dex_stargaze.wasm"
    );
  } catch (error) {
    l(error);
  }

  try {
    ADAPTER_SCHEDULER_COMMON_CONTRACT = getContractByWasm(
      contracts,
      "adapter_scheduler_common.wasm"
    );
  } catch (error) {
    l(error);
  }

  try {
    LENDING_PLATFORM_CONTRACT = getContractByWasm(
      contracts,
      "lending_platform.wasm"
    );
  } catch (error) {
    l(error);
  }

  try {
    MINTER_CONTRACT = getContractByWasm(contracts, "minter.wasm");
  } catch (error) {
    l(error);
  }

  try {
    MINTER_KUJIRA_CONTRACT = getContractByWasm(contracts, "minter_kujira.wasm");
  } catch (error) {
    l(error);
  }

  try {
    ORACLE_CONTRACT = getContractByWasm(contracts, "oracle.wasm");
  } catch (error) {
    l(error);
  }

  return {
    ADAPTER_DEX_STARGAZE_CONTRACT,
    ADAPTER_MARKETPLACE_STARGAZE_CONTRACT,
    ADAPTER_SCHEDULER_COMMON_CONTRACT,
    LENDING_PLATFORM_CONTRACT,
    MINTER_CONTRACT,
    MINTER_KUJIRA_CONTRACT,
    ORACLE_CONTRACT,
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
    ADAPTER_DEX_STARGAZE_CONTRACT,
    ADAPTER_MARKETPLACE_STARGAZE_CONTRACT,
    ADAPTER_SCHEDULER_COMMON_CONTRACT,
    LENDING_PLATFORM_CONTRACT,
    MINTER_CONTRACT,
    MINTER_KUJIRA_CONTRACT,
    ORACLE_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc, owner, signer);
  if (!cwClient) throw new Error("cwClient is not found!");

  const signingClient = cwClient.client as SigningCosmWasmClient;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

  const adapterDexStargazeMsgComposer = new AdapterDexStargazeMsgComposer(
    owner,
    ADAPTER_DEX_STARGAZE_CONTRACT ? ADAPTER_DEX_STARGAZE_CONTRACT.ADDRESS : ""
  );

  const adapterMarketplaceStargazeMsgComposer =
    new AdapterMarketplaceStargazeMsgComposer(
      owner,
      ADAPTER_MARKETPLACE_STARGAZE_CONTRACT
        ? ADAPTER_MARKETPLACE_STARGAZE_CONTRACT.ADDRESS
        : ""
    );

  const adapterSchedulerCommonMsgComposer =
    new AdapterSchedulerCommonMsgComposer(
      owner,
      ADAPTER_SCHEDULER_COMMON_CONTRACT
        ? ADAPTER_SCHEDULER_COMMON_CONTRACT.ADDRESS
        : ""
    );

  const lendingPlatformMsgComposer = new LendingPlatformMsgComposer(
    owner,
    LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : ""
  );

  const minterMsgComposer = new MinterMsgComposer(
    owner,
    MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : ""
  );

  const minterKujiraMsgComposer = new MinterKujiraMsgComposer(
    owner,
    MINTER_KUJIRA_CONTRACT ? MINTER_KUJIRA_CONTRACT.ADDRESS : ""
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
      lendingPlatform,
      adapterMarketplace,
      recoverAddress,
      maxPriceImpact,
    }: {
      admin?: string;
      worker?: string;
      lendingPlatform?: string;
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
          lendingPlatform,
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
      lendingPlatform,
      marketplace,
      infinityPool,
    }: {
      admin?: string;
      worker?: string;
      lendingPlatform?: string;
      marketplace?: string;
      infinityPool?: string;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterMarketplaceStargazeMsgComposer.updateConfig({
          admin,
          worker,
          lendingPlatform,
          marketplace,
          infinityPool,
        }),
      ],
      gasPrice
    );
  }

  async function cwAcceptOffers(
    biddedCollateralItemList: BiddedCollateralItem[],
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterMarketplaceStargazeMsgComposer.acceptOffers({
          biddedCollateralItemList,
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
      lendingPlatform,
      minClockPeriod,
      offchainClock,
    }: {
      admin?: string;
      worker?: string;
      clockSource?: ClockSource;
      lendingPlatform?: string;
      minClockPeriod?: number;
      offchainClock?: string[];
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        adapterSchedulerCommonMsgComposer.updateConfig({
          admin,
          worker,
          lendingPlatform,
          offchainClock,
          clockSource,
          minClockPeriod,
        }),
      ],
      gasPrice
    );
  }

  async function cwPush(targets: LiquidationItem[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [adapterSchedulerCommonMsgComposer.push({ targets })],
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
      marketplace,
      minter,
      oracle,
      worker,
      scheduler,
    }: {
      marketplace?: string;
      minter?: string;
      oracle?: string;
      worker?: string;
      scheduler?: string;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.updateAddressConfig({
          marketplace,
          minter,
          oracle,
          worker,
          scheduler,
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
      isMarketplaceInterfaceEnabled,
      mainCurrency,
      borrowersReserveFractionRatio,
      priceUpdatePeriod,
      unbondingPeriod,
    }: {
      bglCurrency?: CurrencyForTokenUnverified;
      collateralMinValue?: number;
      isMarketplaceInterfaceEnabled?: boolean;
      mainCurrency?: CurrencyForTokenUnverified;
      borrowersReserveFractionRatio?: number;
      priceUpdatePeriod?: number;
      unbondingPeriod?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        lendingPlatformMsgComposer.updateCommonConfig({
          bglCurrency,
          collateralMinValue: collateralMinValue?.toString(),
          isMarketplaceInterfaceEnabled,
          mainCurrency,
          borrowersReserveFractionRatio:
            borrowersReserveFractionRatio?.toString(),
          priceUpdatePeriod,
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

  async function cwLock(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.lock()],
      gasPrice
    );
  }

  async function cwUnlock(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [lendingPlatformMsgComposer.unlock()],
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
      minPriceUpdatePeriod,
    }: {
      admin?: string;
      worker?: string;
      controller?: string[];
      minPriceUpdatePeriod?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        oracleMsgComposer.updateConfig({
          admin,
          worker,
          controller,
          minPriceUpdatePeriod,
        }),
      ],
      gasPrice
    );
  }

  async function cwSetPrices(data: RawPriceItem[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [oracleMsgComposer.setPrices({ data })],
      gasPrice
    );
  }

  async function cwUpdatePrices(data: RawPriceItem[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [oracleMsgComposer.updatePrices({ data })],
      gasPrice
    );
  }

  async function cwRemovePrices(collections: string[], gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [oracleMsgComposer.removePrices({ collections })],
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
      },
    },
    oracle: {
      cwAcceptAdminRole: cwOracleAcceptAdminRole,
      cwUpdateConfig: cwOracleUpdateConfig,
      cwSetPrices,
      cwUpdatePrices,
      cwRemovePrices,
    },
  };
}

async function getCwQueryHelpers(chainId: string, rpc: string) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const {
    ADAPTER_DEX_STARGAZE_CONTRACT,
    ADAPTER_MARKETPLACE_STARGAZE_CONTRACT,
    ADAPTER_SCHEDULER_COMMON_CONTRACT,
    LENDING_PLATFORM_CONTRACT,
    MINTER_CONTRACT,
    MINTER_KUJIRA_CONTRACT,
    ORACLE_CONTRACT,
  } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc);
  if (!cwClient) throw new Error("cwClient is not found!");

  const cosmwasmQueryClient: CosmWasmClient = cwClient.client;

  const adapterDexStargazeQueryClient = new AdapterDexStargazeQueryClient(
    cosmwasmQueryClient,
    ADAPTER_DEX_STARGAZE_CONTRACT ? ADAPTER_DEX_STARGAZE_CONTRACT.ADDRESS : ""
  );

  const adapterMarketplaceStargazeQueryClient =
    new AdapterMarketplaceStargazeQueryClient(
      cosmwasmQueryClient,
      ADAPTER_MARKETPLACE_STARGAZE_CONTRACT
        ? ADAPTER_MARKETPLACE_STARGAZE_CONTRACT.ADDRESS
        : ""
    );

  const adapterSchedulerCommonQueryClient =
    new AdapterSchedulerCommonQueryClient(
      cosmwasmQueryClient,
      ADAPTER_SCHEDULER_COMMON_CONTRACT
        ? ADAPTER_SCHEDULER_COMMON_CONTRACT.ADDRESS
        : ""
    );

  const lendingPlatformQueryClient = new LendingPlatformQueryClient(
    cosmwasmQueryClient,
    LENDING_PLATFORM_CONTRACT ? LENDING_PLATFORM_CONTRACT.ADDRESS : ""
  );

  const minterQueryClient = new MinterQueryClient(
    cosmwasmQueryClient,
    MINTER_CONTRACT ? MINTER_CONTRACT.ADDRESS : ""
  );

  const minterKujiraQueryClient = new MinterKujiraQueryClient(
    cosmwasmQueryClient,
    MINTER_KUJIRA_CONTRACT ? MINTER_KUJIRA_CONTRACT.ADDRESS : ""
  );

  const oracleQueryClient = new OracleQueryClient(
    cosmwasmQueryClient,
    ORACLE_CONTRACT ? ORACLE_CONTRACT.ADDRESS : ""
  );

  // utils

  async function cwQueryOperators(
    ownerAddress: string,
    collectionAddress: string
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
    return logAndReturn(res);
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
    return logAndReturn(res);
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
    return logAndReturn(res);
  }

  async function cwQueryNftOwner(collectionAddress: string, tokenId: string) {
    const queryOwnerOfMsg: QueryOwnerOf = {
      owner_of: { token_id: tokenId },
    };
    const res: OwnerOfResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryOwnerOfMsg
    );
    return logAndReturn(res);
  }

  // adapter-dex-stargaze

  async function cwAdapterDexStargazeQueryConfig() {
    const res = await adapterDexStargazeQueryClient.queryConfig();
    return logAndReturn(res);
  }

  // adapter-marketplace-stargaze

  async function cwAdapterMarketplaceStargazeQueryConfig() {
    const res = await adapterMarketplaceStargazeQueryClient.queryConfig();
    return logAndReturn(res);
  }

  async function cwAdapterMarketplaceStargazeQueryOffersByCollectionAddressList(
    limit: number = 100,
    startAfter?: string
  ) {
    const res =
      await adapterMarketplaceStargazeQueryClient.queryOffersByCollectionAddressList(
        { startAfter, limit }
      );
    return logAndReturn(res);
  }

  // adapter-scheduler-common

  async function cwAdapterSchedulerCommonQueryConfig() {
    const res = await adapterSchedulerCommonQueryClient.queryConfig();
    return logAndReturn(res);
  }

  async function cwAdapterSchedulerCommonQueryLog() {
    const res = await adapterSchedulerCommonQueryClient.queryLog();
    return logAndReturn(res);
  }

  // lending-platform

  async function cwLendingPlatformQueryAddressConfig() {
    const res = await lendingPlatformQueryClient.queryAddressConfig();
    return logAndReturn(res);
  }

  async function cwLendingPlatformQueryRateConfig() {
    const res = await lendingPlatformQueryClient.queryRateConfig();
    return logAndReturn(res);
  }

  async function cwLendingPlatformQueryCommonConfig() {
    const res = await lendingPlatformQueryClient.queryCommonConfig();
    return logAndReturn(res);
  }

  async function cwQueryPlatformRevenue() {
    const res = await lendingPlatformQueryClient.queryPlatformRevenue();
    return logAndReturn(res);
  }

  async function cwQueryBalances() {
    const res = await lendingPlatformQueryClient.queryBalances();
    return logAndReturn(res);
  }

  async function cwQueryUnbonderList(limit: number = 100, startAfter?: string) {
    const res = await lendingPlatformQueryClient.queryUnbonderList({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryUnbonder(address: string) {
    const res = await lendingPlatformQueryClient.queryUnbonder({ address });
    return logAndReturn(res);
  }

  async function cwQueryBorrowerList(limit: number = 100, startAfter?: string) {
    const res = await lendingPlatformQueryClient.queryBorrowerList({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryBorrower(address: string) {
    const res = await lendingPlatformQueryClient.queryBorrower({ address });
    return logAndReturn(res);
  }

  async function cwQueryLiquidatorList(
    limit: number = 100,
    startAfter?: string
  ) {
    const res = await lendingPlatformQueryClient.queryLiquidatorList({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryLiquidator(address: string) {
    const res = await lendingPlatformQueryClient.queryLiquidator({ address });
    return logAndReturn(res);
  }

  async function cwQueryCollateralList(
    limit: number = 100,
    startAfter?: string
  ) {
    const res = await lendingPlatformQueryClient.queryCollateralList({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryCollateral(collectionAddress: string) {
    const res = await lendingPlatformQueryClient.queryCollateral({
      collectionAddress,
    });
    return logAndReturn(res);
  }

  async function cwQueryCollateralByOwner(owner: string) {
    const res = await lendingPlatformQueryClient.queryCollateralByOwner({
      owner,
    });
    return logAndReturn(res);
  }

  async function cwQueryLiquidationBidsByCollectionAddressList(
    limit: number = 100,
    startAfter?: string
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddressList(
        { startAfter, limit }
      );
    return logAndReturn(res);
  }

  async function cwQueryLiquidationBidsByCollectionAddress(address: string) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByCollectionAddress({
        address,
      });
    return logAndReturn(res);
  }

  async function cwQueryLiquidationBidsByLiquidatorAddressList(
    limit: number = 100,
    startAfter?: string
  ) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddressList(
        { startAfter, limit }
      );
    return logAndReturn(res);
  }

  async function cwQueryLiquidationBidsByLiquidatorAddress(address: string) {
    const res =
      await lendingPlatformQueryClient.queryLiquidationBidsByLiquidatorAddress({
        address,
      });
    return logAndReturn(res);
  }

  async function cwQueryProposals(limit: number = 100, startAfter?: number) {
    const res = await lendingPlatformQueryClient.queryProposals({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryCollectionList(
    limit: number = 100,
    startAfter?: string
  ) {
    const res = await lendingPlatformQueryClient.queryCollectionList({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryCollection(address: string) {
    const res = await lendingPlatformQueryClient.queryCollection({
      address,
    });
    return logAndReturn(res);
  }

  async function cwQueryBglCurrencyToMainCurrencyPrice() {
    const res =
      await lendingPlatformQueryClient.queryBglCurrencyToMainCurrencyPrice();
    return logAndReturn(res);
  }

  async function cwQueryConditionalDepositApr(amountToDeposit: number) {
    const res = await lendingPlatformQueryClient.queryConditionalDepositApr({
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
    });
    return logAndReturn(res);
  }

  async function cwQueryLtvList(limit: number = 100, startAfter?: string) {
    const res = await lendingPlatformQueryClient.queryLtvList({
      startAfter,
      limit,
    });
    return logAndReturn(res);
  }

  async function cwQueryConditionalLtv(
    borrower: string,
    amountToDeposit: number = 0,
    amountToBorrow: number = 0
  ) {
    const res = await lendingPlatformQueryClient.queryConditionalLtv({
      borrower,
      amountToDeposit: amountToDeposit ? amountToDeposit.toString() : undefined,
      amountToBorrow: amountToBorrow ? amountToBorrow.toString() : undefined,
    });
    return logAndReturn(res);
  }

  async function cwQueryTotalAvailableToBorrowLiquidity() {
    const res =
      await lendingPlatformQueryClient.queryTotalAvailableToBorrowLiquidity();
    return logAndReturn(res);
  }

  async function cwQueryAvailableToBorrow(
    borrower: string,
    targetLtv?: number
  ) {
    const res = await lendingPlatformQueryClient.queryAvailableToBorrow({
      borrower,
      targetLtv: targetLtv ? targetLtv.toString() : undefined,
    });
    return logAndReturn(res);
  }

  async function cwQueryAmounts() {
    const res = await lendingPlatformQueryClient.queryAmounts();
    return logAndReturn(res);
  }

  // minter

  async function cwQueryCurrenciesByCreator(creator: string) {
    const res = await minterQueryClient.queryCurrenciesByCreator({ creator });
    return logAndReturn(res);
  }

  async function cwMinterQueryConfig() {
    const res = await minterQueryClient.queryConfig();
    return logAndReturn(res);
  }

  async function cwMinterQueryTokenOwner(denom: string) {
    const res = await minterQueryClient.queryTokenOwner({ denom });
    return logAndReturn(res);
  }

  // minter-kujira

  async function cwMinterKujiraQueryCurrenciesByCreator(creator: string) {
    const res = await minterKujiraQueryClient.queryCurrenciesByCreator({
      creator,
    });
    return logAndReturn(res);
  }

  async function cwMinterKujiraQueryConfig() {
    const res = await minterKujiraQueryClient.queryConfig();
    return logAndReturn(res);
  }

  async function cwMinterKujiraQueryTokenOwner(denom: string) {
    const res = await minterKujiraQueryClient.queryTokenOwner({ denom });
    return logAndReturn(res);
  }

  // oracle

  async function cwOracleQueryConfig() {
    const res = await oracleQueryClient.queryConfig();
    return logAndReturn(res);
  }

  async function cwQueryPrices(
    limit: number = 100,
    startAfter?: string,
    collectionAddresses: string[] = []
  ) {
    const msg_1 = {
      startAfter,
      limit,
    };
    const msg_2 = { collectionAddresses };

    const res = await oracleQueryClient.queryPrices(
      collectionAddresses.length ? msg_2 : msg_1
    );
    return logAndReturn(res);
  }

  async function cwQueryBlockTime() {
    const res = await oracleQueryClient.queryBlockTime();
    return logAndReturn(res);
  }

  async function cwOracleQueryLog() {
    const res = await oracleQueryClient.queryLog();
    return logAndReturn(res);
  }

  return {
    utils: {
      cwQueryOperators,
      cwQueryApprovals,
      cwQueryBalanceInNft,
      cwQueryNftOwner,
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
          cwQueryOffersByCollectionAddressList:
            cwAdapterMarketplaceStargazeQueryOffersByCollectionAddressList,
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
      cwQueryUnbonderList,
      cwQueryUnbonder,
      cwQueryBorrowerList,
      cwQueryBorrower,
      cwQueryLiquidatorList,
      cwQueryLiquidator,
      cwQueryCollateralList,
      cwQueryCollateral,
      cwQueryCollateralByOwner,
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
      cwQueryAmounts,
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
      cwQueryBlockTime,
      cwQueryLog: cwOracleQueryLog,
    },
  };
}

export { getCwExecHelpers, getCwQueryHelpers };
