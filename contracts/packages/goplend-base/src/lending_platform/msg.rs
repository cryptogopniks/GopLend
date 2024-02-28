use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::{
    assets::{Currency, Token, TokenUnverified},
    lending_platform::types::{Collateral, Collection, CollectionInfo, Proposal},
};

use super::types::{Borrower, LiquidationBid, Liquidator, Unbonder};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub oracle: Option<String>,
    pub main_currency: Currency<TokenUnverified>,
    pub bgl_currency: Option<Currency<TokenUnverified>>,
    pub cw20_base_code_id: Option<u64>,
    pub unbonding_period: Option<u64>,
    pub borrow_apr: Option<Decimal>,
    pub borrow_fee_rate: Option<Decimal>,
    pub liquidation_fee_rate: Option<Decimal>,
    pub discount_min_rate: Option<Decimal>,
    pub discount_max_rate: Option<Decimal>,
    pub bid_min_rate: Option<Decimal>,
    pub pagination_amount: Option<u32>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum SudoMsg {
    // x/cron
    BeginBlock {},

    EndBlock {},
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

    // admin
    UpdateConfig {
        owner: Option<String>,
        oracle: Option<String>,
        main_currency: Option<Currency<TokenUnverified>>,
        bgl_currency: Option<Currency<TokenUnverified>>,
        cw20_base_code_id: Option<u64>,
        unbonding_period: Option<u64>,
        borrow_apr: Option<Decimal>,
        borrow_fee_rate: Option<Decimal>,
        liquidation_fee_rate: Option<Decimal>,
        discount_min_rate: Option<Decimal>,
        discount_max_rate: Option<Decimal>,
        bid_min_rate: Option<Decimal>,
        pagination_amount: Option<u32>,
    },

    // admin, owner
    CreateBglToken {},

    // TODO: add update metadata
    //
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

// TODO: add estimation queries
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::lending_platform::types::Config)]
    QueryConfig {},

    #[returns(Vec<crate::assets::Funds<crate::assets::Token>>)]
    QueryPlatformRevenue {},

    #[returns(crate::lending_platform::types::Balances)]
    QueryBalances {},

    #[returns(Vec<QueryUnbondersResponseItem>)]
    QueryUnbonders { addresses: Option<Vec<String>> },

    #[returns(Vec<QueryBorrowersResponseItem>)]
    QueryBorrowers { addresses: Option<Vec<String>> },

    #[returns(Vec<QueryLiquidatorsResponseItem>)]
    QueryLiquidators { addresses: Option<Vec<String>> },

    #[returns(Vec<QueryCollateralsResponseItem>)]
    QueryCollaterals {
        collection_addresses: Option<Vec<String>>,
    },

    #[returns(Vec<QueryLiquidationBidsResponseItem>)]
    QueryLiquidationBidsByCollectionAddresses { addresses: Option<Vec<String>> },

    #[returns(Vec<QueryLiquidationBidsResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddresses { addresses: Option<Vec<String>> },

    #[returns(Vec<QueryProposalsResponseItem>)]
    QueryProposals { last_amount: Option<u32> },

    #[returns(Vec<QueryCollectionsResponseItem>)]
    QueryCollections { addresses: Option<Vec<String>> },

    #[returns(Decimal)]
    QueryBglCurrencyToMainCurrencyPrice {},

    #[returns(Decimal)]
    QueryDepositApr {},

    #[returns(Decimal)]
    QueryLtv { borrower: String },

    // test
    #[returns(u64)]
    QueryBlockCounter {},
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
pub struct QueryLiquidationBidsResponseItem {
    pub address: Addr,
    pub liquidation_bids: Vec<LiquidationBid>,
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
