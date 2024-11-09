use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::assets::{Currency, Token};

#[cw_serde]
pub struct RawPriceItem {
    pub collection_address: String,
    pub price: Uint128,
    pub usd_price: Uint128,
}

// NFT price
#[cw_serde]
pub struct PriceItem<T> {
    pub address: Addr,
    pub price: T,
    pub usd_price: T,
    pub update_date: u64,
    pub is_outdated: bool,
}

// Main currency price
#[cw_serde]
pub struct PriceRatioItem {
    pub value: Decimal, // price impact must be included
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
    pub main_currency: Currency<Token>,
    pub stable_currency: Currency<Token>,
}

#[cw_serde]
pub struct Log {
    pub block_time: u64,
    pub sender: Addr,
    pub outdated_price_collections: Vec<Addr>,
    pub counter: u8,
}
