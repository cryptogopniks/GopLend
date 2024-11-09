use cosmwasm_std::{coin, Addr, Deps, Env, StdResult};

use goplend_base::marketplace::state::{PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT};
use sg_index_query::QueryOptions;
use stargaze_marketplace_v2::{
    msg::PriceOffset,
    orders::{Bid, CollectionBid, OrderDetails},
    state::{Config, Denom, TokenId},
};

use crate::state::{COLLECTION_OFFERS, ROYALTY_REGISTRY};

pub fn query_collection_bids_by_price(
    deps: Deps,
    _env: Env,
    collection: String,
    denom: Denom,
    query_options: Option<QueryOptions<PriceOffset>>,
) -> StdResult<Vec<CollectionBid>> {
    let limit = query_options
        .map(|x| x.limit.unwrap_or(PAGINATION_DEFAULT_LIMIT))
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    Ok(COLLECTION_OFFERS
        .load(deps.storage)?
        .into_iter()
        .filter(|x| x.collection == collection)
        .take(limit)
        .map(|x| CollectionBid {
            id: x.id,
            creator: x.bidder,
            collection: x.collection,
            details: OrderDetails {
                price: coin(x.price.u128(), denom.clone()),
                finder: None,
                recipient: None,
            },
        })
        .collect())
}

pub fn query_bids_by_token_price(
    deps: Deps,
    _env: Env,
    collection: String,
    token_id: TokenId,
    denom: Denom,
    query_options: Option<QueryOptions<PriceOffset>>,
) -> StdResult<Vec<Bid>> {
    let limit = query_options
        .map(|x| x.limit.unwrap_or(PAGINATION_DEFAULT_LIMIT))
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    Ok(COLLECTION_OFFERS
        .load(deps.storage)?
        .into_iter()
        .filter(|x| x.collection == collection)
        .take(limit)
        .map(|x| Bid {
            id: x.id,
            creator: x.bidder,
            collection: x.collection,
            token_id: token_id.to_string(),
            details: OrderDetails {
                price: coin(x.price.u128(), denom.clone()),
                finder: None,
                recipient: None,
            },
        })
        .collect())
}

pub fn config(deps: Deps, env: Env) -> StdResult<Config<Addr>> {
    let royalty_registry = ROYALTY_REGISTRY.load(deps.storage)?;

    Ok(Config {
        fee_manager: env.contract.address,
        royalty_registry,
        protocol_fee_bps: 2,
        max_royalty_fee_bps: 100,
        maker_reward_bps: 0,
        taker_reward_bps: 0,
        default_denom: "ustars".to_string(),
    })
}
