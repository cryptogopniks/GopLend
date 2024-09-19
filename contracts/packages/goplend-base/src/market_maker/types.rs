use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use sg_marketplace::state::CollectionBid;

use crate::assets::Token;

#[cw_serde]
pub struct CollectionOwners<T: ToString> {
    pub collection_address: T,
    pub owner_list: Vec<T>,
}

#[cw_serde]
pub struct LiquidityInfo {
    pub collection_address: Addr,
    pub amount_undistributed: Uint128,
    pub offers: Vec<CollectionBid>, // to get amount_in_offers
}

impl LiquidityInfo {
    pub fn new_empty(collection_address: &Addr) -> Self {
        Self {
            collection_address: collection_address.to_owned(),
            amount_undistributed: Uint128::zero(),
            offers: vec![],
        }
    }
}

#[cw_serde]
pub struct OffersResponse {
    pub collection_address: Addr,
    pub price_and_outpost_list: Vec<(Uint128, Addr)>,
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
