use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

use crate::adapters::dex::stargaze::types::Route;

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub lending_platform_stable: Option<String>,
    pub adapter_marketplace: Option<String>,
    pub recover_address: Option<String>,
    pub max_price_impact: Option<Decimal>,
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
        lending_platform_stable: Option<String>,
        adapter_marketplace: Option<String>,
        recover_address: Option<String>,
        max_price_impact: Option<Decimal>,
    },

    // adapter-marketplace
    Swap {
        route: Route,
        price_ratio: Decimal, // price impact must be included
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},
}
