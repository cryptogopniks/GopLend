use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::assets::{Funds, TokenUnverified};

#[cw_serde]
pub struct RawPriceItem {
    pub collection_address: String,
    pub raw_prices: Vec<Funds<TokenUnverified>>,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub scheduler: Option<Addr>,
}

impl Config {
    pub fn new(admin: &Addr, scheduler: &Option<Addr>) -> Self {
        Self {
            admin: admin.to_owned(),
            scheduler: scheduler.to_owned(),
        }
    }
}
