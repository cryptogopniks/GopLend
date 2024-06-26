use cosmwasm_std::Addr;
use cw_storage_plus::{Deque, Item, Map};

use crate::{
    adapters::scheduler::common::types::Log,
    oracle::types::{Config, TransferAdminState},
};

use super::types::{PriceItem, PriceRatioItem};

pub const CONTRACT_NAME: &str = "goplend-oracle";

pub const EXECUTION_COOLDOWN: u64 = 4;
pub const MAX_PRICE_UPDATE_PERIOD: u64 = 30;
pub const LOG_COUNTER: u8 = 0;

pub const PAGINATION_MAX_LIMIT: u32 = 10_000;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 1_000;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const PRICE_RATIO: Item<PriceRatioItem> = Item::new("price ratio");
pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");

pub const PRICE_AND_DATE_LIST: Map<&Addr, PriceItem> = Map::new("price and data list");

pub const LOG: Deque<Log> = Deque::new("log");
