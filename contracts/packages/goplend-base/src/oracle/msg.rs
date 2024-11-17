use cosmwasm_schema::{cw_serde, QueryResponses};

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
    UpdatePrices {
        /// list of (collection_address_postfix, usd_price_with_decimals_2)
        data: Vec<(String, u32)>,
    },

    RemovePrices {
        collections: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(super::types::QueryPricesResponse)]
    QueryPrices {
        amount: u32,
        start_after: Option<String>,
        collections: Option<Vec<String>>,
    },

    #[returns(u64)]
    QueryBlockTime {},
}
