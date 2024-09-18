use cw_storage_plus::Item;

use crate::market_maker::types::OutpostConfig;

pub const CONTRACT_NAME: &str = "goplend-market-maker-outpost";

pub const PAGINATION_MAX_LIMIT: u32 = 100;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 10;

pub const EXPIRATION_DELAY_IN_DAYS: u64 = 90;
pub const FINDERS_FEE_BPS: u64 = 0;

pub const CONFIG: Item<OutpostConfig> = Item::new("config");
