use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::market_maker::types::{HubConfig, LiquidityInfo, TransferAdminState};

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
/// collection liquidity by collection address
pub const LIQUIDITY: Map<&Addr, LiquidityInfo> = Map::new("liquidity");
