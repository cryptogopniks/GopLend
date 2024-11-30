use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

// NFT price
#[cw_serde]
pub struct PriceItem {
    pub collection: Addr,
    pub price: Uint128,
}

impl PriceItem {
    pub fn new(collection: impl ToString, price: u128) -> Self {
        Self {
            collection: Addr::unchecked(collection.to_string()),
            price: Uint128::new(price),
        }
    }
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

pub fn split_address(address: impl ToString) -> (String, String) {
    let address = address.to_string();
    let (prefix, postfix) = address.split_once('1').unwrap();
    (prefix.to_string(), postfix.to_string())
}

pub fn get_collection(prefix: &str, postfix: &str) -> Addr {
    Addr::unchecked(format!("{}1{}", prefix, postfix))
}
