use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Funds, Token},
    oracle::types::Config,
};

pub const CONTRACT_NAME: &str = "crates.io:cryptogopniks-oracle";

pub const CONFIG: Item<Config> = Item::new("config");

pub const PRICE_AND_DATE_LIST: Map<&Addr, (Funds<Token>, u64)> = Map::new("price and data list");
