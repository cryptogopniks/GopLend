use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::market_maker::types::{HubConfig, TransferAdminState};

pub const CONTRACT_NAME: &str = "goplend-market-maker-hub";

pub const ADD_OUTPOST_REPLY: u64 = 1;

pub const PAGINATION_MAX_LIMIT: u32 = 100;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 10;
pub const MAX_OUTPOST_AMOUNT: u8 = 10;
pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const IS_LOCKED: Item<bool> = Item::new("is locked");
pub const CONFIG: Item<HubConfig> = Item::new("config");
pub const OUTPOSTS: Item<Vec<Addr>> = Item::new("outposts");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");

/// collection owners by collection address
pub const OWNERS: Map<&Addr, Vec<Addr>> = Map::new("owners by collection");
/// undistributed collection liquidity by collection address
pub const UNDISTRIBUTED_LIQUIDITY: Map<&Addr, Uint128> = Map::new("undistributed_liquidity");
