import { RawPriceItem } from "../codegen/Oracle.types";
import { ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem, CollectionInfoForString, CurrencyForTokenUnverified, LiquidationItem, ProposalForStringAndTokenUnverified } from "../codegen/LendingPlatform.types";
import { DirectSecp256k1HdWallet, OfflineSigner, OfflineDirectSigner } from "@cosmjs/proto-signing";
import { ClockSource } from "../codegen/AdapterSchedulerKujira.types";
import { InstantiateMarketingInfo, Logo, Metadata } from "../codegen/Minter.types";
import { TokenUnverified, QueryAllOperatorsResponse, ApprovalsResponse, TokensResponse, OwnerOfResponse } from "../interfaces";
declare function getCwExecHelpers(chainId: string, rpc: string, owner: string, signer: (OfflineSigner & OfflineDirectSigner) | DirectSecp256k1HdWallet): Promise<{
    utils: {
        cwRevoke: (collectionAddress: string, senderAddress: string, operator: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
    };
    adapter: {
        scheduler: {
            kujira: {
                cwUpdateConfig: ({ clockSource, lendingPlatform, minClockPeriod, offchainClock, }: {
                    clockSource?: ClockSource | undefined;
                    lendingPlatform?: string | undefined;
                    minClockPeriod?: number | undefined;
                    offchainClock?: string[] | undefined;
                }, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
                cwPush: (targets: LiquidationItem[], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
            };
        };
    };
    lending: {
        cwDeposit: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUnbond: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwWithdraw: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwWithdrawCollateral: (collections: CollectionInfoForString[], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwApproveAndDepositCollateral: (senderAddress: string, operator: string, collections: CollectionInfoForString[], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwBorrow: (amount: number, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwRepay: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwPlaceBid: (collections: CollectionInfoForString[], discount: number, amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwRemoveBid: (collections: CollectionInfoForString[], creationDate: number, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateBid: (collections: CollectionInfoForString[], creationDate: number, amount: number, discount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwLiquidate: (targets: LiquidationItem[], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateAddressConfig: ({ marketplace, minter, oracle, owner, scheduler, }: {
            marketplace?: string | undefined;
            minter?: string | undefined;
            oracle?: string | undefined;
            owner?: string | undefined;
            scheduler?: string | undefined;
        }, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateRateConfig: ({ bidMinRate, borrowApr, borrowFeeRate, discountMaxRate, discountMinRate, liquidationFeeRate, }: {
            bidMinRate?: number | undefined;
            borrowApr?: number | undefined;
            borrowFeeRate?: number | undefined;
            discountMaxRate?: number | undefined;
            discountMinRate?: number | undefined;
            liquidationFeeRate?: number | undefined;
        }, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateCommonConfig: ({ bglCurrency, collateralMinValue, isMarketplaceInterfaceEnabled, mainCurrency, minAvailableLiquidityMultiplier, priceUpdatePeriod, unbondingPeriod, }: {
            bglCurrency?: CurrencyForTokenUnverified | undefined;
            collateralMinValue?: number | undefined;
            isMarketplaceInterfaceEnabled?: boolean | undefined;
            mainCurrency?: CurrencyForTokenUnverified | undefined;
            minAvailableLiquidityMultiplier?: number | undefined;
            priceUpdatePeriod?: number | undefined;
            unbondingPeriod?: number | undefined;
        }, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwDepositReserveLiquidity: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwReinforceBglToken: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwLock: (gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUnlock: (gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwDistributeFunds: (addressAndWeightList: [string, number][], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwRemoveCollection: (address: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwCreateProposal: (proposal: ProposalForStringAndTokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwRejectProposal: (id: number, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwAcceptProposal: (id: number, amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
    };
    minter: {
        cwCreateNative: (tokenOwner: string, subdenom: string, decimals: number, paymentAmount: number, paymentDenom: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwCreateCw20: (tokenOwner: string, name: string, symbol: string, marketing: InstantiateMarketingInfo, decimals: number | undefined, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwMint: (amount: number, recipient: string, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwBurn: (amount: number, token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwSetMetadataNative: (token: TokenUnverified, metadata: Metadata, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateMarketingCw20: (token: TokenUnverified, description: string, marketing: string, project: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUploadLogoCw20: (token: TokenUnverified, logo: Logo, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwChangeAdminNative: (token: TokenUnverified, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateMinterCw20: (token: TokenUnverified, newMinter: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwRegisterCurrency: (currency: CurrencyForTokenUnverified, creator: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdateConfig: ({ cw20BaseCodeId, owner, }: {
            cw20BaseCodeId?: number | undefined;
            owner?: string | undefined;
        }, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
    };
    oracle: {
        cwUpdateConfig: (scheduler: string, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
        cwUpdatePrices: (data: RawPriceItem[], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
    };
}>;
declare function getCwQueryHelpers(chainId: string, rpc: string): Promise<{
    utils: {
        cwQueryOperators: (collectionAddress: string, ownerAddress: string) => Promise<QueryAllOperatorsResponse>;
        cwQueryApprovals: (collectionAddress: string, tokenId: string) => Promise<ApprovalsResponse>;
        cwQueryBalanceInNft: (owner: string, collectionAddress: string) => Promise<TokensResponse>;
        cwQueryNftOwner: (collectionAddress: string, tokenId: string) => Promise<OwnerOfResponse>;
    };
    adapter: {
        scheduler: {
            kujira: {
                cwQueryConfig: () => Promise<import("../codegen/AdapterSchedulerKujira.types").Config>;
                cwQueryLog: (amount: number) => Promise<import("../codegen/AdapterSchedulerKujira.types").ArrayOfTupleOfUint64AndUint32>;
            };
        };
    };
    lending: {
        cwQueryAddressConfig: () => Promise<import("../codegen/LendingPlatform.types").AddressConfig>;
        cwQueryRateConfig: () => Promise<import("../codegen/LendingPlatform.types").RateConfig>;
        cwQueryCommonConfig: () => Promise<import("../codegen/LendingPlatform.types").CommonConfig>;
        cwQueryPlatformRevenue: () => Promise<import("../codegen/LendingPlatform.types").ArrayOfFundsForToken>;
        cwQueryBalances: () => Promise<import("../codegen/LendingPlatform.types").Balances>;
        cwQueryUnbonderList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryUnbondersResponseItem>;
        cwQueryUnbonder: (address: string) => Promise<import("../codegen/LendingPlatform.types").Unbonder>;
        cwQueryBorrowerList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryBorrowersResponseItem>;
        cwQueryBorrower: (address: string) => Promise<import("../codegen/LendingPlatform.types").Borrower>;
        cwQueryLiquidatorList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryLiquidatorsResponseItem>;
        cwQueryLiquidator: (address: string) => Promise<import("../codegen/LendingPlatform.types").Liquidator>;
        cwQueryCollateralList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryCollateralsResponseItem>;
        cwQueryCollateral: (collectionAddress: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfCollateral>;
        cwQueryLiquidationBidsByCollectionAddressList: (limit?: number, startAfter?: string) => Promise<ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem>;
        cwQueryLiquidationBidsByCollectionAddress: (address: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfLiquidationBid>;
        cwQueryLiquidationBidsByLiquidatorAddressList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryLiquidationBidsByLiquidatorAddressListResponseItem>;
        cwQueryLiquidationBidsByLiquidatorAddress: (address: string) => Promise<ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem>;
        cwQueryProposals: (limit?: number, startAfter?: number) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryProposalsResponseItem>;
        cwQueryCollectionList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfQueryCollectionsResponseItem>;
        cwQueryCollection: (address: string) => Promise<import("../codegen/LendingPlatform.types").CollectionForAddr>;
        cwQueryBglCurrencyToMainCurrencyPrice: () => Promise<string>;
        cwQueryConditionalDepositApr: (amountToDeposit: number) => Promise<string>;
        cwQueryLtvList: (limit?: number, startAfter?: string) => Promise<import("../codegen/LendingPlatform.types").ArrayOfTupleOfAddrAndDecimal>;
        cwQueryConditionalLtv: (borrower: string, amountToDeposit?: number, amountToBorrow?: number) => Promise<string>;
        cwQueryTotalAvailableToBorrowLiquidity: () => Promise<string>;
        cwQueryAvailableToBorrow: (borrower: string, targetLtv?: number) => Promise<string>;
        cwQueryAmounts: () => Promise<import("../codegen/LendingPlatform.types").QueryAmountsResponse>;
    };
    minter: {
        cwQueryCurrenciesByCreator: (creator: string) => Promise<import("../codegen/Minter.types").QueryCurrenciesFromCreatorResponse>;
        cwQueryConfig: () => Promise<import("../codegen/Minter.types").Config>;
    };
    oracle: {
        cwQueryConfig: () => Promise<import("../codegen/Oracle.types").Config>;
        cwQueryPrices: (limit?: number, startAfter?: string, collectionAddresses?: string[]) => Promise<import("../codegen/Oracle.types").ArrayOfPriceItem>;
        cwQueryBlockTime: () => Promise<number>;
    };
}>;
export { getCwExecHelpers, getCwQueryHelpers };
