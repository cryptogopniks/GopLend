use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::assets::Token;

#[cw_serde]
pub struct CollectionOwner<T: ToString> {
    pub collection_address: T,
    pub owner: T,
}

#[cw_serde]
pub struct LiquidityInfo {
    pub collection_address: Addr,
    pub amount_undistributed: Uint128,
    pub price_list: Vec<Uint128>,
}

impl LiquidityInfo {
    pub fn new_empty(collection_address: &Addr) -> Self {
        Self {
            collection_address: collection_address.to_owned(),
            amount_undistributed: Uint128::zero(),
            price_list: vec![],
        }
    }
}

#[cw_serde]
pub struct OffersResponse {
    pub collection_address: Addr,
    pub price_list: Vec<Uint128>,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub controller: Option<Vec<Addr>>,
    pub lending_platform: Option<Addr>,
    pub oracle: Option<Addr>,
    pub token: Token,
}
