use cw_storage_plus::{Deque, Item};

use crate::adapters::scheduler::kujira::types::{ClockSource, Config, TransferAdminState};

pub const CONTRACT_NAME: &str = "cryptogopniks-adapter-scheduler-kujira";

pub const CLOCK_SOURCE: ClockSource = ClockSource::Offchain;
pub const MIN_CLOCK_PERIOD: u64 = 3;
pub const LOG_COUNTER: u32 = 0;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");

pub const LOG: Deque<(u64, u32)> = Deque::new("block_time and counter log");
