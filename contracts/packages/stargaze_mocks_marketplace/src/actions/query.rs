use cosmwasm_std::{Deps, Env, StdResult};

use sg_marketplace::{
    msg::{Bidder, Collection, CollectionBidOffset, CollectionBidsResponse, CollectionOffset},
    state::CollectionBid,
};

use crate::state::{COLLECTION_OFFERS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT};

pub fn collection_bids_sorted_by_price(
    deps: Deps,
    env: Env,
    collection: Collection,
    _start_after: Option<CollectionBidOffset>,
    limit: Option<u32>,
) -> StdResult<CollectionBidsResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let bids: Vec<CollectionBid> = COLLECTION_OFFERS
        .load(deps.storage)?
        .into_iter()
        .filter(|x| x.collection.to_string() == collection)
        .take(limit)
        .map(|x| CollectionBid {
            collection: x.collection,
            bidder: x.bidder,
            price: x.price,
            finders_fee_bps: None,
            expires_at: env.block.time.plus_days(1_000),
        })
        .collect();

    Ok(CollectionBidsResponse { bids })
}

pub fn collection_bids_by_bidder(
    deps: Deps,
    env: Env,
    bidder: Bidder,
    _start_after: Option<CollectionOffset>,
    limit: Option<u32>,
) -> StdResult<CollectionBidsResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let bids: Vec<CollectionBid> = COLLECTION_OFFERS
        .load(deps.storage)?
        .into_iter()
        .filter(|x| x.bidder.to_string() == bidder)
        .take(limit)
        .map(|x| CollectionBid {
            collection: x.collection,
            bidder: x.bidder,
            price: x.price,
            finders_fee_bps: None,
            expires_at: env.block.time.plus_days(1_000),
        })
        .collect();

    Ok(CollectionBidsResponse { bids })
}
