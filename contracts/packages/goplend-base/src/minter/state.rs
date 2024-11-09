use cosmwasm_std::Addr;
use cw_storage_plus::{Deque, Item, Map};

use crate::minter::types::{Config, CurrencyInfo, FaucetConfig, TransferAdminState};

pub const CONTRACT_NAME: &str = "minter";

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 3600;

pub const SAVE_CW20_ADDRESS_REPLY: u64 = 0;

pub const DEFAULT_DECIMALS: u8 = 6;
pub const MAX_TOKENS_PER_OWNER: u16 = 10;

/// Stores user functions pause flag
pub const IS_PAUSED: Item<bool> = Item::new("is_paused");
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the state of changing admin process
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer_admin_state");
/// Stores the state of changing token owner process
pub const TRANSFER_OWNER_STATE: Item<Vec<(String, TransferAdminState)>> =
    Item::new("denom_or_address_and_transfer_owner_state");

pub const FAUCET_CONFIG: Map<&str, FaucetConfig> = Map::new("faucet_config");
pub const CURRENCIES: Map<&str, CurrencyInfo> = Map::new("currency_info_by_denom_or_address");
pub const TOKEN_COUNT: Map<&Addr, u16> = Map::new("token_count_by_owner");
/// last claim date by (user, denom_or_address)
pub const LAST_CLAIM_DATE: Map<(&Addr, &str), u64> = Map::new("last_claim_date");

pub const TEMPORARY_CURRENCY: Deque<CurrencyInfo> = Deque::new("temporary_currency");
