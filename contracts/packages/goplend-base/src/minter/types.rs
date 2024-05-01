use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::assets::{Currency, Token};

#[cw_serde]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

#[cw_serde]
pub struct Metadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: Option<String>,
    pub uri_hash: Option<String>,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub cw20_base_code_id: Option<u64>,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}

#[cw_serde]
pub struct QueryCurrenciesFromCreatorResponse {
    pub currencies: Vec<Currency<Token>>,
}
