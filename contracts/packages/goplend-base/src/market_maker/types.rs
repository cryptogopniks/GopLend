use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::{assets::Token, lending_platform::types::CollectionInfo};

#[cw_serde]
pub struct CollectionOwners<T: ToString> {
    pub collection_address: T,
    pub owner_list: Vec<T>,
}

#[cw_serde]
pub struct LiquidityInfo {
    pub collection_address: Addr,
    pub amount_undistributed: Uint128,
    pub price_and_outpost_list: Vec<(Uint128, Addr)>,
}

impl LiquidityInfo {
    pub fn new_empty(collection_address: &Addr) -> Self {
        Self {
            collection_address: collection_address.to_owned(),
            amount_undistributed: Uint128::zero(),
            price_and_outpost_list: vec![],
        }
    }
}

#[cw_serde]
pub struct OffersResponse {
    pub collection_address: Addr,
    pub price_and_outpost_list: Vec<(Uint128, Addr)>,
}

#[cw_serde]
pub struct CollateralResponse {
    pub outpost: Addr,
    pub collection_info: CollectionInfo<Addr>,
}

#[cw_serde]
pub struct HubConfig {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub marketplace: Option<Addr>,
    pub controller: Option<Vec<Addr>>,
    pub token: Token,
    pub max_outpost_amount: u8,
    pub outpost_code_id: Option<u64>,
    // pub boundary_high: Decimal,
    // pub boundary_low: Decimal,
}

#[cw_serde]
pub struct OutpostConfig {
    pub hub: Addr,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
