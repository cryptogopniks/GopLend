use cosmwasm_std::Addr;
use cw_storage_plus::{Deque, Item, Map};

use crate::{
    assets::{Currency, Token},
    minter::types::{Config, TransferAdminState},
};

pub const CONTRACT_NAME: &str = "goplend-minter";

pub const CW20_CODE_ID: u64 = 1;
pub const SAVE_CW20_ADDRESS_REPLY: u64 = 0;

pub const CW20_NAME: &str = "bonded GopLend STARS";
pub const CW20_SYMBOL: &str = "bglSTARS";
pub const DEFAULT_DECIMALS: u8 = 6;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");

pub const OWNERS: Map<&str, (Currency<Token>, Addr)> = Map::new("currency and owner by denom");

pub const OWNER_AND_DECIMALS: Deque<(Addr, u8)> = Deque::new("owner and decimals");
