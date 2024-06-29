use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::faucet::types::Config;

pub const CONTRACT_NAME: &str = "goplend-faucet";

pub const CLAIMABLE_AMOUNT: u128 = 10_000_000_000;
pub const CLAIM_COOLDOWN: u64 = 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOKEN: Item<String> = Item::new("token");

pub const LAST_CLAIM_DATE: Map<&Addr, u64> = Map::new("last claim date");
