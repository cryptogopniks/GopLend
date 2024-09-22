use cosmwasm_std::{Decimal, Deps, Env, StdResult, Uint128};

use cw_utils::Duration;
use goplend_base::converters::str_to_dec;
use sg_marketplace::{
    msg::{
        BidOffset, BidsResponse, Collection, CollectionBidOffset, CollectionBidsResponse,
        ParamsResponse,
    },
    state::{Bid, CollectionBid, SudoParams},
    ExpiryRange,
};

use crate::state::{COLLECTION_OFFERS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT};

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
            token_id: 5,
            finders_fee_bps: None,
            expires_at: env.block.time.plus_days(1_000),
        })
        .collect();

    Ok(BidsResponse { bids })
}

pub fn params(_deps: Deps, _env: Env) -> StdResult<ParamsResponse> {
    Ok(ParamsResponse {
        params: SudoParams {
            trading_fee_percent: str_to_dec("0.02"),
            ask_expiry: ExpiryRange { min: 0, max: 100 },
            bid_expiry: ExpiryRange { min: 0, max: 100 },
            operators: vec![],
            max_finders_fee_percent: str_to_dec("0.1"),
            min_price: Uint128::zero(),
            stale_bid_duration: Duration::Time(100),
            bid_removal_reward_percent: Decimal::zero(),
            listing_fee: Uint128::zero(),
        },
    })
}
