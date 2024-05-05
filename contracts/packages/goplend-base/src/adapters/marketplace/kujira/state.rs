use cw_storage_plus::Item;

use crate::adapters::marketplace::kujira::types::{Config, TransferAdminState};

pub const CONTRACT_NAME: &str = "cryptogopniks-adapter-marketplace-kujira";

pub const PAGINATION_MAX_LIMIT: u32 = 10_000;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 1_000;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");
