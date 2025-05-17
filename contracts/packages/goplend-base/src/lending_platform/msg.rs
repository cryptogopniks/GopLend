use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_gopniks::assets::{Currency, Cw20ReceiveMsg, TokenUnverified};

use crate::lending_platform::types::{CollectionInfo, LiquidationItem, Proposal};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub oracle: Option<String>,
    pub minter: Option<String>,
    pub scheduler: Option<String>,
    pub market_maker: Option<String>,

    pub borrow_apr: Option<Decimal>,
    pub borrow_fee_rate: Option<Decimal>,
    pub liquidation_fee_rate: Option<Decimal>,
    pub discount_min_rate: Option<Decimal>,
    pub discount_max_rate: Option<Decimal>,
    pub bid_min_rate: Option<Decimal>,

    pub main_currency: Currency<TokenUnverified>,
    pub bgl_currency: Option<Currency<TokenUnverified>>,
    pub unbonding_period: Option<u64>,
    pub collateral_min_value: Option<Uint128>,
    pub borrowers_reserve_fraction_ratio: Option<Decimal>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    // depositors
    Deposit {},

    Unbond {},

    Rebond {
        creation_date: u64,
    },

    Withdraw {},

    // borrowers, liquidators
    WithdrawCollateral {
        collections: Vec<CollectionInfo<String>>,
    },

    // borrowers
    DepositCollateral {
        collections: Vec<CollectionInfo<String>>,
    },

    Borrow {
        amount: Uint128,
    },

    Repay {},

    // liquidators
    PlaceBid {
        collections: Vec<CollectionInfo<String>>,
        discount: Decimal,
    },

    UpdateBid {
        collections: Vec<CollectionInfo<String>>,
        creation_date: u64,
        amount: Uint128, // amount is required to decrease bid
        discount: Decimal,
    },

    RemoveBid {
        collection_addresses: Vec<String>,
        creation_date: u64,
    },

    // scheduler
    Liquidate {
        targets: Vec<LiquidationItem>,
    },

    // any
    AcceptAdminRole {},

    // admin
    UpdateAddressConfig {
        admin: Option<String>,
        worker: Option<String>,
        oracle: Option<String>,
        minter: Option<String>,
        scheduler: Option<String>,
        market_maker: Option<String>,
    },

    UpdateRateConfig {
        borrow_apr: Option<Decimal>,
        borrow_fee_rate: Option<Decimal>,
        liquidation_fee_rate: Option<Decimal>,
        discount_min_rate: Option<Decimal>,
        discount_max_rate: Option<Decimal>,
        bid_min_rate: Option<Decimal>,
    },

    UpdateCommonConfig {
        main_currency: Option<Currency<TokenUnverified>>,
        bgl_currency: Option<Currency<TokenUnverified>>,
        unbonding_period: Option<u64>,
        collateral_min_value: Option<Uint128>,
        borrowers_reserve_fraction_ratio: Option<Decimal>,
    },

    // admin, worker
    DepositReserveLiquidity {},

    WithdrawReserveLiquidity {
        amount: Uint128,
    },

    ReinforceBglToken {},

    Pause {},

    Unpause {},

    DistributeFunds {
        address_and_weight_list: Vec<(String, Decimal)>,
    },

    RemoveCollection {
        address: String,
    },

    CreateProposal {
        proposal: Proposal<String, TokenUnverified>,
    },

    RejectProposal {
        id: u32,
    },

    // collections creators
    AcceptProposal {
        id: u32,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::AddressConfig)]
    QueryAddressConfig {},

    #[returns(super::types::RateConfig)]
    QueryRateConfig {},

    #[returns(super::types::CommonConfig)]
    QueryCommonConfig {},

    #[returns(Vec<super::types::Funds<cw_gopniks::assets::Token> >)]
    QueryPlatformRevenue {},

    #[returns(super::types::Balances)]
    QueryBalances {},

    #[returns(Vec<super::types::QueryUnbondersResponseItem>)]
    QueryUnbonderList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::Unbonder)]
    QueryUnbonder { address: String },

    #[returns(Vec<super::types::QueryBorrowersResponseItem>)]
    QueryBorrowerList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::Borrower)]
    QueryBorrower { address: String },

    #[returns(Vec<super::types::QueryLiquidatorsResponseItem>)]
    QueryLiquidatorList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::Liquidator)]
    QueryLiquidator { address: String },

    #[returns(Vec<super::types::QueryCollateralsResponseItem>)]
    QueryCollateralList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(Vec<super::types::Collateral>)]
    QueryCollateral { collection_address: String },

    #[returns(Vec<super::types::QueryCollateralsResponseItem>)]
    QueryCollateralByOwner { owner: String },

    #[returns(Vec<super::types::QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryLiquidationBidsByCollectionAddressList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(Vec<super::types::LiquidationBid>)]
    QueryLiquidationBidsByCollectionAddress { address: String },

    #[returns(Vec<super::types::QueryLiquidationBidsByLiquidatorAddressListResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddressList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(Vec<super::types::QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddress { address: String },

    #[returns(Vec<super::types::QueryProposalsResponseItem>)]
    QueryProposals {
        amount: u32,
        start_after: Option<u32>,
    },

    #[returns(Vec<super::types::QueryCollectionsResponseItem>)]
    QueryCollectionList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::Collection<Addr>)]
    QueryCollection { address: String },

    #[returns(Decimal)]
    QueryBglCurrencyToMainCurrencyPrice {},

    #[returns(Decimal)]
    QueryConditionalDepositApr {
        amount_to_deposit: Option<Uint128>,
        amount_to_withdraw: Option<Uint128>,
    },

    #[returns(Vec<(Addr, Option<Decimal>)>)]
    QueryLtvList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(Decimal)]
    QueryConditionalLtv {
        borrower: String,
        amount_to_deposit: Option<Uint128>,
        amount_to_withdraw: Option<Uint128>,
        amount_to_borrow: Option<Uint128>,
        amount_to_repay: Option<Uint128>,
    },

    #[returns(Uint128)]
    QueryTotalAvailableToBorrowLiquidity {},

    #[returns(Uint128)]
    QueryAvailableToBorrow {
        borrower: String,
        target_ltv: Option<Decimal>,
    },

    #[returns(super::types::QueryAmountsResponse)]
    QueryAmounts {},

    #[returns(super::types::QueryUserInfoResponse)]
    QueryUserInfo { address: String },
}
