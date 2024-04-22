use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::{
    assets::{Currency, Token, TokenUnverified},
    lending_platform::types::{
        Borrower, Collateral, Collection, CollectionInfo, LiquidationBid, LiquidationItem,
        Liquidator, Proposal, Unbonder,
    },
};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub oracle: Option<String>,
    pub minter: Option<String>,
    pub scheduler: Option<String>,
    pub marketplace: Option<String>,

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
    pub price_update_period: Option<u64>,
    pub is_marketplace_interface_enabled: Option<bool>,
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

    RemoveBid {
        collections: Vec<CollectionInfo<String>>,
        creation_date: u64,
    },

    UpdateBid {
        collections: Vec<CollectionInfo<String>>,
        creation_date: u64,
        amount: Uint128, // amount is required to decrease bid
        discount: Decimal,
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
        marketplace: Option<String>,
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
        price_update_period: Option<u64>,
        is_marketplace_interface_enabled: Option<bool>,
    },

    // admin, worker
    DepositReserveLiquidity {},

    ReinforceBglToken {},

    Lock {},

    Unlock {},

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
    #[returns(crate::lending_platform::types::AddressConfig)]
    QueryAddressConfig {},

    #[returns(crate::lending_platform::types::RateConfig)]
    QueryRateConfig {},

    #[returns(crate::lending_platform::types::CommonConfig)]
    QueryCommonConfig {},

    #[returns(Vec<crate::assets::Funds<crate::assets::Token>>)]
    QueryPlatformRevenue {},

    #[returns(crate::lending_platform::types::Balances)]
    QueryBalances {},

    #[returns(Vec<QueryUnbondersResponseItem>)]
    QueryUnbonderList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::lending_platform::types::Unbonder)]
    QueryUnbonder { address: String },

    #[returns(Vec<QueryBorrowersResponseItem>)]
    QueryBorrowerList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::lending_platform::types::Borrower)]
    QueryBorrower { address: String },

    #[returns(Vec<QueryLiquidatorsResponseItem>)]
    QueryLiquidatorList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::lending_platform::types::Liquidator)]
    QueryLiquidator { address: String },

    #[returns(Vec<QueryCollateralsResponseItem>)]
    QueryCollateralList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Vec<crate::lending_platform::types::Collateral>)]
    QueryCollateral { collection_address: String },

    #[returns(Vec<QueryCollateralsResponseItem>)]
    QueryCollateralByOwner { owner: String },

    #[returns(Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryLiquidationBidsByCollectionAddressList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Vec<crate::lending_platform::types::LiquidationBid>)]
    QueryLiquidationBidsByCollectionAddress { address: String },

    #[returns(Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddressList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddress { address: String },

    #[returns(Vec<QueryProposalsResponseItem>)]
    QueryProposals {
        start_after: Option<u32>,
        limit: Option<u32>,
    },

    #[returns(Vec<QueryCollectionsResponseItem>)]
    QueryCollectionList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::lending_platform::types::Collection<cosmwasm_std::Addr>)]
    QueryCollection { address: String },

    #[returns(Decimal)]
    QueryBglCurrencyToMainCurrencyPrice {},

    #[returns(Decimal)]
    QueryConditionalDepositApr { amount_to_deposit: Option<Uint128> },

    #[returns(Vec<(Addr, Decimal)>)]
    QueryLtvList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Decimal)]
    QueryConditionalLtv {
        borrower: String,
        amount_to_deposit: Option<Uint128>,
        amount_to_borrow: Option<Uint128>,
    },

    #[returns(Uint128)]
    QueryTotalAvailableToBorrowLiquidity {},

    #[returns(Uint128)]
    QueryAvailableToBorrow {
        borrower: String,
        target_ltv: Option<Decimal>,
    },

    #[returns(QueryAmountsResponse)]
    QueryAmounts {},
}

#[cw_serde]
pub struct QueryUnbondersResponseItem {
    pub address: Addr,
    pub unbonder: Unbonder,
}

#[cw_serde]
pub struct QueryBorrowersResponseItem {
    pub address: Addr,
    pub borrower: Borrower,
}

#[cw_serde]
pub struct QueryLiquidatorsResponseItem {
    pub address: Addr,
    pub liquidator: Liquidator,
}

#[cw_serde]
pub struct QueryCollateralsResponseItem {
    pub address: Addr,
    pub collateral: Vec<Collateral>,
}

#[cw_serde]
pub struct QueryLiquidationBidsByCollectionAddressListResponseItem {
    pub collection_address: Addr,
    pub liquidation_bids: Vec<LiquidationBid>,
}

#[cw_serde]
pub struct QueryLiquidationBidsByLiquidatorAddressListResponseItem {
    pub liquidator_address: Addr,
    pub collection_and_liquidation_bids_list:
        Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>,
}

#[cw_serde]
pub struct QueryProposalsResponseItem {
    pub id: u32,
    pub proposal: Proposal<Addr, Token>,
}

#[cw_serde]
pub struct QueryCollectionsResponseItem {
    pub address: Addr,
    pub collection: Collection<Addr>,
}

#[cw_serde]
pub struct QueryAmountsResponse {
    pub unbonders: u32,
    pub borrowers: u32,
    pub collaterals: u32,
    pub liquidators: u32,
    pub liquidation_bids: u32,
    pub proposals: u32,
    pub collections: u32,
}
