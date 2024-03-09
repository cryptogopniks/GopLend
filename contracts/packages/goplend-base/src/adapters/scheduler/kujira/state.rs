use cw_storage_plus::{Deque, Item};

use super::types::Config;

pub const CONTRACT_NAME: &str = "cryptogopniks-adapter-scheduler-kujira";

pub const CONFIG: Item<Config> = Item::new("config");

pub const LOG: Deque<(u64, u32)> = Deque::new("block_time and counter log");
