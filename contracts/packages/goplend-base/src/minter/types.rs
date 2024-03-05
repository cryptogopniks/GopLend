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
    pub owner: Option<Addr>,
    pub cw20_base_code_id: Option<u64>,
}

impl Config {
    pub fn new(admin: &Addr, owner: &Option<Addr>, cw20_base_code_id: &Option<u64>) -> Self {
        Self {
            admin: admin.to_owned(),
            owner: owner.to_owned(),
            cw20_base_code_id: cw20_base_code_id.to_owned(),
        }
    }
}

#[cw_serde]
pub struct QueryCurrenciesFromCreatorResponse {
    pub denoms: Vec<Currency<Token>>,
}
