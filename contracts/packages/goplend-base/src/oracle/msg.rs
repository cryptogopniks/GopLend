use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

use crate::oracle::types::RawPriceItem;

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub controller: Option<Vec<String>>,
    pub execution_cooldown: Option<u64>,
    pub max_price_update_period: Option<u64>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // any
    AcceptAdminRole {},

    // admin
    UpdateConfig {
        admin: Option<String>,
        worker: Option<String>,
        controller: Option<Vec<String>>,
        execution_cooldown: Option<u64>,
        max_price_update_period: Option<u64>,
    },

    // controller
    SetPrices {
        // price_ratio must be decreased offchain due to price_impact
        price_ratio: Decimal,
        data: Vec<RawPriceItem>,
    },

    UpdatePrices {
        // price_ratio must be decreased offchain due to price_impact
        price_ratio: Decimal,
        data: Vec<RawPriceItem>,
    },

    RemovePrices {
        collections: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::oracle::types::Config)]
    QueryConfig {},

    #[returns(Vec<crate::oracle::types::PriceItem>)]
    QueryPrices {
        collection_addresses: Option<Vec<String>>,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::oracle::types::PriceRatioItem)]
    QueryPriceRatio {},

    #[returns(u64)]
    QueryBlockTime {},

    #[returns(Vec<crate::adapters::scheduler::common::types::Log>)]
    QueryLog {},
}

// TODO: add offers per token in v2
// #[cw_serde]
// pub struct QueryPricesResponseItem {
//     pub address: Addr,
//     pub currency: Currency<Token>,
//     pub token_and_price_list: Vec<(String, Uint128)>,
//     pub collection_price: Uint128,
//     pub price_update_date: u64,
// }
