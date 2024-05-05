use cw_storage_plus::Item;

use crate::adapters::marketplace::kujira::types::{Config, TransferAdminState};

pub const CONTRACT_NAME: &str = "cryptogopniks-adapter-marketplace-kujira";

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");
