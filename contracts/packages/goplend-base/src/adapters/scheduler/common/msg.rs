use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::{
    adapters::{dex::stargaze::types::Route, scheduler::common::types::ClockSource},
    lending_platform::types::LiquidationItem,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub lending_platform_main: Option<String>,
    pub lending_platform_stable: Option<String>,
    pub offchain_clock: Option<Vec<String>>,
    pub clock_source: Option<ClockSource>,
    pub execution_cooldown: Option<u64>,
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
        lending_platform_main: Option<String>,
        lending_platform_stable: Option<String>,
        offchain_clock: Option<Vec<String>>,
        clock_source: Option<ClockSource>,
        execution_cooldown: Option<u64>,
    },

    // scheduler
    Push {
        targets: Vec<LiquidationItem>,
        route: Route,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<super::types::Log>)]
    QueryLog {},
}
