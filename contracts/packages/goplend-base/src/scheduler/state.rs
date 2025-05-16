use cw_storage_plus::{Deque, Item};

use crate::scheduler::types::{Config, Log};

pub const CONTRACT_NAME: &str = "goplend-scheduler";

pub const EXECUTION_COOLDOWN: u64 = 4;
pub const LOG_COUNTER: u8 = 0;

pub const CONFIG: Item<Config> = Item::new("config");

pub const LOG: Deque<Log> = Deque::new("log");
