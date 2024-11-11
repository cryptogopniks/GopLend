use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::{
    assets::{Currency, TokenUnverified},
    oracle::types::RawPriceItem,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub controller: Option<Vec<String>>,
    pub execution_cooldown: Option<u64>,
    pub max_price_update_period: Option<u64>,
    pub stable_currency: Currency<TokenUnverified>,
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
        stable_currency: Option<Currency<TokenUnverified>>,
    },

    // controller
    SetPrices(Vec<RawPriceItem>),

    UpdatePrices(Vec<RawPriceItem>),

    RemovePrices {
        collections: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<super::types::PriceItem<crate::assets::Funds<crate::assets::Token>>>)]
    QueryPrices {
        amount: u32,
        start_after: Option<String>,
        collections: Option<Vec<String>>,
    },

    #[returns(u64)]
    QueryBlockTime {},

    #[returns(Vec<super::types::Log>)]
    QueryLog {},
}
