use cosmwasm_std::Addr;
use cw_storage_plus::{Deque, Item, Map};

use crate::{
    assets::{Currency, Token},
    minter::types::Config,
};

pub const CONTRACT_NAME: &str = "cryptogopniks-minter";

pub const SAVE_CW20_ADDRESS_REPLY: u64 = 0;

pub const DEFAULT_DECIMALS: u8 = 6;

pub const CONFIG: Item<Config> = Item::new("config");

pub const OWNERS: Map<&str, (Currency<Token>, Addr)> = Map::new("currency and owner by denom");

pub const OWNER_AND_DECIMALS: Deque<(Addr, u8)> = Deque::new("owner and decimals");
