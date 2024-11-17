use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

// NFT price
#[cw_serde]
pub struct PriceItem {
    pub collection: Addr,
    pub price: Uint128,
}

#[cw_serde]
pub struct QueryPricesResponse {
    pub is_outdated: bool,
    pub data: Vec<PriceItem>,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Addr,
    pub controller: Vec<Addr>,
    pub execution_cooldown: u64,
    pub max_price_update_period: u64,
}
