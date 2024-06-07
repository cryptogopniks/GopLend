use cosmwasm_std::{Deps, Env, StdResult};

use sg_marketplace::{
    msg::{BidOffset, BidsResponse, Collection, CollectionBidOffset, CollectionBidsResponse},
    state::{Bid, CollectionBid},
};

use crate::state::{COLLECTION_OFFERS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT};

// TODO: sort bids
pub fn collection_bids_sorted_by_price(
    deps: Deps,
    env: Env,
    collection: Collection,
    _start_before: Option<CollectionBidOffset>,
    limit: Option<u32>,
) -> StdResult<CollectionBidsResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let bids: Vec<CollectionBid> = COLLECTION_OFFERS
        .load(deps.storage)?
        .into_iter()
        .filter(|x| x.collection == collection)
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

pub fn bids_sorted_by_price(
    deps: Deps,
    env: Env,
    collection: Collection,
    _start_before: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let bids: Vec<Bid> = COLLECTION_OFFERS
        .load(deps.storage)?
        .into_iter()
        .filter(|x| x.collection == collection)
        .take(limit)
        .map(|x| Bid {
            collection: x.collection,
            bidder: x.bidder,
            price: x.price,
            token_id: 5, // TODO
            finders_fee_bps: None,
            expires_at: env.block.time.plus_days(1_000),
        })
        .collect();

    Ok(BidsResponse { bids })
}
