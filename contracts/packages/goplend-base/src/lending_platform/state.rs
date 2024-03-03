use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Funds, Token},
    lending_platform::types::{
        Balances, Borrower, Collateral, Collection, Config, LiquidationBid, Liquidator,
        PaginationConfig, Proposal, Unbonder,
    },
};

pub const CONTRACT_NAME: &str = "crates.io:cryptogopniks-lending-platform";

pub const MAIN_CURRENCY_DECIMALS: u8 = 6;
pub const BGL_CURRENCY_DECIMALS: u8 = 6;

pub const CW20_CODE_ID: u64 = 1;
pub const CW20_NAME: &str = "bonded GopLend STARS";
pub const CW20_SYMBOL: &str = "bglSTARS";
pub const SAVE_CW20_ADDRESS_REPLY: u64 = 0;

pub const YEAR_IN_SECONDS: u64 = 365 * 24 * 3600;
pub const UNBONDING_PERIOD: u64 = 14 * 24 * 3600;
pub const BORROW_APR: &str = "0.25";
pub const BORROW_FEE_RATE: &str = "0.05";
pub const LIQUIDATION_FEE_RATE: &str = "0.05";
pub const DISCOUNT_MIN_RATE: &str = "0";
pub const DISCOUNT_MAX_RATE: &str = "0.2";
pub const BID_MIN_RATE: &str = "0.1";

pub const MAX_DEPOSIT_APR: &str = "100"; // 10_000 %

pub const LIQUIDATION_BID_MAX_TOKEN_AMOUNT: usize = 5;
pub const LIQUIDATION_BID_MAX_AMOUNT: usize = 5;

pub const PAGINATION_AMOUNT: u32 = 10;
pub const PAGINATION_INDEX: Option<Addr> = None;

pub const IS_LOCKED: Item<bool> = Item::new("is locked");
pub const CONFIG: Item<Config> = Item::new("config");
pub const PAGINATION_CONFIG: Item<PaginationConfig> = Item::new("pagination config");
pub const BALANCES: Item<Balances> = Item::new("balances");
pub const PLATFORM_REVENUE: Item<Vec<Funds<Token>>> = Item::new("platform revenue");
pub const PROPOSAL_COUNTER: Item<u32> = Item::new("proposal counter");

pub const UNBONDERS: Map<&Addr, Unbonder> = Map::new("unbonder by address");
pub const BORROWERS: Map<&Addr, Borrower> = Map::new("borrower by address");
pub const COLLATERALS: Map<&Addr, Vec<Collateral>> = Map::new("collaterals by collection address");
pub const LIQUIDATORS: Map<&Addr, Liquidator> = Map::new("liquidator by address");
pub const LIQUIDATION_BIDS: Map<&Addr, Vec<LiquidationBid>> =
    Map::new("liquidation bids by collection address");
pub const PROPOSALS: Map<u32, Proposal<Addr, Token>> = Map::new("proposal by id");
pub const COLLECTIONS: Map<&Addr, Collection<Addr>> = Map::new("collection by address");

// for tests
pub const BLOCK_COUNTER: Item<u64> = Item::new("block counter");
