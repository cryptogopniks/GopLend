use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Currency, Token},
    minter::types::Config,
};

pub const CONTRACT_NAME: &str = "cryptogopniks-minter";

pub const SAVE_CW20_ADDRESS_REPLY: u64 = 0;

pub const CONFIG: Item<Config> = Item::new("config");

pub const CURRENCIES: Map<&Addr, Vec<Currency<Token>>> = Map::new("currencies by owner address");
