use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Currency, Token},
    minter_kujira::types::Config,
};

pub const CONTRACT_NAME: &str = "cryptogopniks-minter-kujira";

pub const DEFAULT_DECIMALS: u8 = 6;

pub const CONFIG: Item<Config> = Item::new("config");

pub const OWNERS: Map<&str, (Currency<Token>, Addr)> = Map::new("currency and owner by denom");
