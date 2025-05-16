use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{lending_platform::types::CollectionInfo, market_maker::types::Config};

pub const CONTRACT_NAME: &str = "goplend-market-maker";

/// Stores user functions pause flag
pub const IS_PAUSED: Item<bool> = Item::new("is_paused");
pub const CONFIG: Item<Config> = Item::new("config");

/// collection offers sorted in descending order by collection address
pub const OFFERS: Map<&Addr, Vec<Uint128>> = Map::new("offers_by_collection");
/// collection owner by collection address
pub const OWNERS: Map<&Addr, Addr> = Map::new("owners_by_collection");
/// liquidated collaterals by collection owner address
pub const COLLATERALS: Map<&Addr, Vec<CollectionInfo<Addr>>> =
    Map::new("collaterals_by_maker_address");
/// undistributed collection liquidity by collection address
pub const UNDISTRIBUTED_LIQUIDITY: Map<&Addr, Uint128> = Map::new("undistributed_liquidity");
