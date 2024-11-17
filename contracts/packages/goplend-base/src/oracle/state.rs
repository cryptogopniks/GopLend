use cw_storage_plus::{Item, Map};

use crate::{lending_platform::types::TransferAdminState, oracle::types::Config};

pub const CONTRACT_NAME: &str = "goplend-oracle";

pub const EXECUTION_COOLDOWN: u64 = 4;
pub const MAX_PRICE_UPDATE_PERIOD: u64 = 90;
pub const DECIMALS_MULTIPLIER: u128 = 10_000;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer_admin_state");

pub const UPDATE_DATE: Item<u64> = Item::new("update_date");

/// usd_price_with_decimals_2 by collection_address_postfix                                                 \
/// for example                                                                                             \
/// stars1f37v0rdvrred27tlqqcpkrqpzfv6ddr2a97zzu -> neutron1f37v0rdvrred27tlqqcpkrqpzfv6ddr2dxqan2          \
/// neutron1f37v0rdvrred27tlqqcpkrqpzfv6ddr2dxqan2 -> f37v0rdvrred27tlqqcpkrqpzfv6ddr2dxqan2                \
/// 100_000_510_000 uusdc -> 100_000.51 $ -> 10000051
pub const PRICES: Map<&str, u32> = Map::new("prices");
