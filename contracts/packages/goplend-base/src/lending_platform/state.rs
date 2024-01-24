use cw_storage_plus::Item;

use crate::lending_platform::types::Config;

pub const CONTRACT_NAME: &str = "crates.io:cryptogopniks-lending-platform";

pub const IS_LOCKED: Item<bool> = Item::new("is_locked");
pub const CONFIG: Item<Config> = Item::new("config");
