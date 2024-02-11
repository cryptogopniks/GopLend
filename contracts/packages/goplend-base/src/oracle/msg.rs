use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::{
    assets::{Currency, Funds, Token},
    oracle::types::RawPriceItem,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub scheduler: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // admin
    UpdateConfig { scheduler: Option<String> },

    // scheduler
    UpdatePrices { data: Vec<RawPriceItem> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::oracle::types::Config)]
    QueryConfig {},

    #[returns(Vec<QueryPricesResponseItem>)]
    QueryPrices {
        collection_addresses: Option<Vec<String>>,
    },
    // TODO: add block_time query
}

#[cw_serde]
pub struct QueryPricesResponseItem {
    pub address: Addr,
    pub price: Funds<Token>,
    pub price_update_date: u64,
}

// TODO: use
// #[cw_serde]
// pub struct QueryPricesResponseItem {
//     pub address: Addr,
//     pub currency: Currency<Token>,
//     pub token_and_price_list: Vec<(String, Uint128)>,
//     pub collection_price: Uint128,
//     pub price_update_date: u64,
// }
