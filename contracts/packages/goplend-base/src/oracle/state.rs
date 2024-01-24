use cw_storage_plus::Item;

use crate::oracle::types::Config;

pub const CONTRACT_NAME: &str = "crates.io:cryptogopniks-oracle";

pub const CONFIG: Item<Config> = Item::new("config");
