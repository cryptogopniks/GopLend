use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};

use crate::assets::{Funds, Token, TokenUnverified};

#[cw_serde]
pub struct RawPriceItem {
    pub collection_address: String,
    pub price: Funds<TokenUnverified>,
    pub usd_price: Funds<TokenUnverified>,
}

// NFT price
#[cw_serde]
pub struct PriceItem {
    pub address: Addr,
    pub price: Funds<Token>,
    pub usd_price: Funds<Token>,
    pub update_date: u64,
    pub is_outdated: bool,
}

// Main currency price
#[cw_serde]
pub struct PriceRatioItem {
    pub value: Decimal, // price impact must be included
    pub update_date: u64,
    pub is_outdated: bool,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub controller: Option<Vec<Addr>>,
    pub execution_cooldown: u64,
    pub max_price_update_period: u64,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
