use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub lending_platform: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // admin
    UpdateConfig {
        owner: Option<String>,
        lending_platform: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::oracle::types::Config)]
    QueryConfig {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}
