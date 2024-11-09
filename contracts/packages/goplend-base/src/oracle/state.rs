use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Deque, Item, Map};

use crate::{
    lending_platform::types::TransferAdminState,
    oracle::types::{Config, Log, PriceItem, PriceRatioItem},
};

pub const CONTRACT_NAME: &str = "goplend-oracle";

pub const EXECUTION_COOLDOWN: u64 = 4;
pub const MAX_PRICE_UPDATE_PERIOD: u64 = 30;
pub const LOG_COUNTER: u8 = 0;

pub const PRICE_RATIO: Item<PriceRatioItem> = Item::new("price_ratio");
pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer_admin_state");

pub const PRICE_AND_DATE_LIST: Map<&Addr, PriceItem<Uint128>> = Map::new("price_and_data_list");

pub const LOG: Deque<Log> = Deque::new("log");
