use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    assets::{Funds, Token},
    lending_platform::types::{
        Balances, Borrower, Collateral, Collection, Config, Depositor, LiquidationBid, Liquidator,
        Proposal,
    },
};

pub const CONTRACT_NAME: &str = "crates.io:cryptogopniks-lending-platform";

pub const UNBONDING_PERIOD: u64 = 14 * 24 * 3600;
pub const BORROW_APR: &str = "0.25";
pub const BORROW_FEE_RATE: &str = "0.05";
pub const LIQUIDATION_FEE_RATE: &str = "0.05";
pub const DISCOUNT_MIN_RATE: &str = "0";
pub const DISCOUNT_MAX_RATE: &str = "0.2";
pub const BID_MIN_RATE: &str = "0.1";

pub const IS_LOCKED: Item<bool> = Item::new("is locked");
pub const CONFIG: Item<Config> = Item::new("config");
pub const BALANCES: Item<Balances> = Item::new("balances");
pub const PLATFORM_REVENUE: Item<Vec<Funds<Token>>> = Item::new("platform revenue");
pub const PROPOSAL_COUNTER: Item<u32> = Item::new("proposal counter");

pub const DEPOSITORS: Map<&Addr, Depositor> = Map::new("depositor by address");
pub const BORROWERS: Map<&Addr, Borrower> = Map::new("borrower by address");
pub const COLLATERALS: Map<&Addr, Collateral> = Map::new("collateral by collection address");
pub const LIQUIDATORS: Map<&Addr, Liquidator> = Map::new("liquidator by address");
pub const LIQUIDATION_BIDS: Map<&Addr, Vec<LiquidationBid>> =
    Map::new("liquidation bids by collection address");
pub const PROPOSALS: Map<u32, Proposal<Addr, Token>> = Map::new("proposal by id");
pub const COLLECTIONS: Map<&Addr, Collection<Addr>> = Map::new("collection by address");
