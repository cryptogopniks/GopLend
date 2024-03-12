use cosmwasm_schema::{cw_serde, QueryResponses};

use super::types::ClockSource;

#[cw_serde]
pub struct InstantiateMsg {
    pub lending_platform: Option<String>,
    pub offchain_clock: Option<Vec<String>>,
    pub clock_source: Option<ClockSource>,
    pub min_clock_period: Option<u64>,
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
    // admin
    UpdateConfig {
        lending_platform: Option<String>,
        offchain_clock: Option<Vec<String>>,
        clock_source: Option<ClockSource>,
        min_clock_period: Option<u64>,
    },

    // scheduler
    Push {
        borrowers: Option<Vec<String>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<(u64,u32)>)]
    QueryLog { amount: u32 },
}
