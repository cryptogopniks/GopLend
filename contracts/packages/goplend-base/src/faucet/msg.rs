use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub subdenom: String,
    pub claimable_amount: Option<Uint128>,
    pub claim_cooldown: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        claimable_amount: Option<Uint128>,
        claim_cooldown: Option<u64>,
    },

    Claim {},

    ClaimFor {
        address_and_amount_list: Vec<(String, Uint128)>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(String)]
    QueryDenom {},

    #[returns(u64)]
    QueryLastClaimDate { address: String },
}
