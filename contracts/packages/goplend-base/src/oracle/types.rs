use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::assets::{Currency, Token};

#[cw_serde]
pub struct RawPriceItem {
    pub collection: String,
    pub usd_price: Uint128,
}

// NFT price
#[cw_serde]
pub struct PriceItem<T> {
    pub address: Addr,
    pub usd_price: T,
    pub update_date: u64,
    pub is_outdated: bool,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub controller: Option<Vec<Addr>>,
    pub execution_cooldown: u64,
    pub max_price_update_period: u64,
    pub stable_currency: Currency<Token>,
}

#[cw_serde]
pub struct Log {
    pub block_time: u64,
    pub sender: Addr,
    pub outdated_price_collections: Vec<Addr>,
    pub counter: u8,
}
