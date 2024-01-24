use cosmwasm_schema::{cw_serde, QueryResponses};

use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub oracle: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // admin
    UpdateConfig {
        owner: Option<String>,
        oracle: Option<String>,
    },

    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::lending_platform::types::Config)]
    QueryConfig {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}
