use cw_storage_plus::Item;

use super::types::Config;

pub const CONTRACT_NAME: &str = "cryptogopniks-adapter-marketplace-kujira";

pub const CONFIG: Item<Config> = Item::new("config");
