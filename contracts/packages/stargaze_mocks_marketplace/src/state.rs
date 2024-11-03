use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use goplend_base::assets::Token;
use stargaze_marketplace_v2::state::OrderId;

pub const CONTRACT_NAME: &str = "stargaze-mocks-markeplace";

pub const PAGINATION_MAX_LIMIT: u32 = 10_000;
pub const PAGINATION_DEFAULT_LIMIT: u32 = 1_000;

#[cw_serde]
pub struct Bid {
    pub id: OrderId,
    pub collection: Addr,
    pub bidder: Addr,
    pub price: Uint128,
}

pub const TOKEN: Item<Token> = Item::new("token");
pub const COLLECTION_OFFERS: Item<Vec<Bid>> = Item::new("collection offers");
pub const ROYALTY_REGISTRY: Item<Addr> = Item::new("royalty registry");
