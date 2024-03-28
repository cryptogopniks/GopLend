use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::oracle::types::RawPriceItem;

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

    #[returns(Vec<crate::oracle::types::PriceItem>)]
    QueryPrices {
        collection_addresses: Option<Vec<String>>,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(u64)]
    QueryBlockTime {},
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
