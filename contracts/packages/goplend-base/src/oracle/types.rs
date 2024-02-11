use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::assets::{Currency, TokenUnverified};

// TODO: add offers per token
#[cw_serde]
pub struct RawPriceItem {
    pub collection_address: String,
    pub currency: Currency<TokenUnverified>,
    pub raw_prices: Vec<Uint128>,
    pub trading_volume_per_day: Uint128,
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
