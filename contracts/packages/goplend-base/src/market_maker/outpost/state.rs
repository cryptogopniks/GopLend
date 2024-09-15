use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::market_maker::types::OutpostConfig;

pub const CONTRACT_NAME: &str = "goplend-market-maker-outpost";

pub const PAGINATION_MAX_LIMIT: u32 = 100;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 10;

pub const CONFIG: Item<OutpostConfig> = Item::new("config");
