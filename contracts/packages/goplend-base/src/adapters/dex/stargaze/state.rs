use cw_storage_plus::Item;

use crate::adapters::dex::stargaze::types::{Config, TransferAdminState};

pub const CONTRACT_NAME: &str = "goplend-adapter-dex-stargaze";

// ----------------------------------------- osmosis -----------------------------------------
pub const SWAP_VENUE_OSMOSIS: &str = "osmosis-poolmanager";

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

// ----------------------------------------- neutron -----------------------------------------
pub const SWAP_VENUE_NEUTRON: &str = "neutron-astroport";
pub const CONTRACT_NEUTRON_SKIP_ENTRY_POINT: &str =
    "neutron1h70czc4nz6enrcwpuz5nhxwupmmsgfx4mk4mncnffpwwjma3d8qshtjfr4";

pub const POOL_NEUTRON_STARS_NTRN: &str =
    "neutron15th0f33w9rq35la0aqe66w8hmlr07wy5arap736x4rv836pddn0s8322fg"; // 0.45 %
pub const POOL_NEUTRON_NTRN_USDC: &str =
    "neutron1q3rfmflhp79rz0sdjxdgf7t3x5twce9az6n0fegspkjstrax42kq0g96rn"; // 0.45 %

pub const DENOM_NEUTRON_STARS: &str =
    "ibc/A139C0E0B5E87CBA8EAEEB12B9BEE13AC7C814CFBBFA87BBCADD67E31003466C";
pub const DENOM_NEUTRON_NTRN: &str = "untrn";
pub const DENOM_NEUTRON_USDC: &str =
    "ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81";

pub const CHANNEL_STARGAZE_NEUTRON: &str = "channel-191";
pub const CHANNEL_NEUTRON_NOBLE: &str = "channel-30";

// ----------------------------------------- test -----------------------------------------
pub const POOL_OSMOSIS_NTRN_OSMO: u32 = 1388;

pub const DENOM_OSMOSIS_NTRN: &str =
    "ibc/126DA09104B71B164883842B769C0E9EC1486C0887D27A9999E395C2C8FB5682";

pub const CHANNEL_NEUTRON_OSMOSIS: &str = "channel-10";
pub const CHANNEL_NOBLE_NEUTRON: &str = "channel-18";

// ------------------------------------------------------------------------------------------
pub const PREFIX_OSMOSIS: &str = "osmo";
pub const PREFIX_NEUTRON: &str = "neutron";
pub const PREFIX_NOBLE: &str = "noble";
pub const PREFIX_STARGAZE: &str = "stars";

pub const PORT: &str = "transfer";
pub const CHANNEL_NOBLE_STARGAZE: &str = "channel-11";
pub const IBC_TIMEOUT: u64 = 10 * 60;

pub const MAX_PRICE_IMPACT: &str = "0.05";

pub const PAGINATION_MAX_LIMIT: u32 = 100;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 10;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");
