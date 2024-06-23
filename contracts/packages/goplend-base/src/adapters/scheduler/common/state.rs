use cw_storage_plus::{Deque, Item};

use crate::adapters::scheduler::common::types::{ClockSource, Config, Log, TransferAdminState};

pub const CONTRACT_NAME: &str = "goplend-adapter-scheduler-common";

pub const CLOCK_SOURCE: ClockSource = ClockSource::Offchain;
pub const EXECUTION_COOLDOWN: u64 = 4;
pub const LOG_COUNTER: u8 = 0;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");

pub const LOG: Deque<Log> = Deque::new("log");
