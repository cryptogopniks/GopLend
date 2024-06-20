use cw_storage_plus::Item;

use crate::adapters::dex::stargaze::types::{Config, TransferAdminState};

pub const CONTRACT_NAME: &str = "goplend-adapter-dex-stargaze";

// https://github.com/skip-mev/skip-api-contracts/blob/main/deployed-contracts/osmosis/mainnet.toml#L28
pub const CONTRACT_OSMOSIS_SKIP_ENTRY_POINT: &str =
    "osmo1vkdakqqg5htq5c3wy2kj2geq536q665xdexrtjuwqckpads2c2nsvhhcyv";

// https://api-osmosis.imperator.co/pools/v2/all?low_liquidity=false
pub const POOL_OSMOSIS_STARS_OSMO: u32 = 1635; // 0 %, highest liquidity
pub const POOL_OSMOSIS_OSMO_USDC: u32 = 1263; // 0.05 %, highest liquidity

pub const DENOM_OSMOSIS_STARS: &str =
    "ibc/987C17B11ABC2B20019178ACE62929FE9840202CE79498E29FE8E5CB02B7C0A4";
pub const DENOM_OSMOSIS_OSMO: &str = "uosmo";
pub const DENOM_OSMOSIS_USDC: &str =
    "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4";

pub const CHANNEL_STARGAZE_OSMOSIS: &str = "channel-0";
pub const CHANNEL_OSMOSIS_NOBLE: &str = "channel-750";
pub const CHANNEL_NOBLE_STARGAZE: &str = "channel-11";

pub const IBC_TIMEOUT: u64 = 10 * 60;

pub const MAX_PRICE_IMPACT: &str = "0.05";
// pub const PRICE_UPDATE_PERIOD: u64 = 5 * 60; // in seconds

pub const PAGINATION_MAX_LIMIT: u32 = 100;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 10;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");
