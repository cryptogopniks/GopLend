use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Funds, Token},
    lending_platform::types::{
        AddressConfig, Balances, Borrower, Collateral, Collection, CommonConfig, LiquidationBid,
        Liquidator, Proposal, RateConfig, TransferAdminState, Unbonder,
    },
};

pub const CONTRACT_NAME: &str = "goplend-lending-platform";

pub const MAIN_CURRENCY_DECIMALS: u8 = 6;
pub const BGL_CURRENCY_DECIMALS: u8 = 6;

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
pub const LIQUIDATION_BID_MAX_COLLECTION_AMOUNT: usize = 5;

pub const COLLATERAL_MIN_VALUE: u128 = 500_000_000;

pub const BORROWERS_RESERVE_FRACTION_RATIO: &str = "0.6";

pub const IS_MARKETPLACE_INTERFACE_ENABLED: bool = false;

pub const BLOCK_TIME_OFFSET: u64 = 60;

pub const PAGINATION_MAX_LIMIT: u32 = 10_000;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 1_000;

pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3600;

pub const IS_LOCKED: Item<bool> = Item::new("is locked");
pub const ADDRESS_CONFIG: Item<AddressConfig> = Item::new("address config");
pub const RATE_CONFIG: Item<RateConfig> = Item::new("rate config");
pub const COMMON_CONFIG: Item<CommonConfig> = Item::new("common config");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer admin state");
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
