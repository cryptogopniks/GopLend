use cw_storage_plus::{Deque, Item};

use super::types::{ClockSource, Config};

pub const CONTRACT_NAME: &str = "cryptogopniks-adapter-scheduler-stargaze";

pub const CLOCK_SOURCE: ClockSource = ClockSource::Offchain;
pub const MIN_CLOCK_PERIOD: u64 = 3;
pub const LOG_COUNTER: u32 = 0;

pub const CONFIG: Item<Config> = Item::new("config");

pub const LOG: Deque<(u64, u32)> = Deque::new("block_time and counter log");
