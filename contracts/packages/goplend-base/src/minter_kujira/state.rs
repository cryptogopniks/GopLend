use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Currency, Token},
    minter_kujira::types::{Config, TransferAdminState},
};

pub const CONTRACT_NAME: &str = "goplend-minter-kujira";

pub const DEFAULT_DECIMALS: u8 = 6;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");

pub const OWNERS: Map<&str, (Currency<Token>, Addr)> = Map::new("currency and owner by denom");
