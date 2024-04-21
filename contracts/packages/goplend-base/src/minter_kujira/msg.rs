use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::assets::{Currency, TokenUnverified};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateNative {
        token_owner: String,
        subdenom: String,
        decimals: Option<u8>,
    },

    Mint {
        token: TokenUnverified,
        amount: Uint128,
        recipient: String,
    },

    Burn {},

    // admin
    RegisterCurrency {
        currency: Currency<TokenUnverified>,
        creator: String,
    },

    UpdateConfig {
        owner: Option<String>,
    },

    UpdateTokenOwner {
        denom: String,
        owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::minter_kujira::types::QueryCurrenciesFromCreatorResponse)]
    QueryCurrenciesByCreator { creator: String },

    #[returns(crate::minter_kujira::types::Config)]
    QueryConfig {},

    #[returns(cosmwasm_std::Addr)]
    QueryTokenOwner { denom: String },
}
