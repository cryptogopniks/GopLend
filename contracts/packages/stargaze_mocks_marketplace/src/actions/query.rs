use cosmwasm_std::{Addr, Deps, Env, StdResult};

use stargaze_marketplace_v2::state::Config;

use crate::state::ROYALTY_REGISTRY;

// pub fn collection_bids_sorted_by_price(
//     deps: Deps,
//     env: Env,
//     collection: Collection,
//     _start_before: Option<CollectionBidOffset>,
//     limit: Option<u32>,
// ) -> StdResult<CollectionBidsResponse> {
//     let limit = limit
//         .unwrap_or(PAGINATION_DEFAULT_LIMIT)
//         .min(PAGINATION_MAX_LIMIT) as usize;

//     let bids: Vec<CollectionBid> = COLLECTION_OFFERS
//         .load(deps.storage)?
//         .into_iter()
//         .filter(|x| x.collection == collection)
//         .take(limit)
//         .map(|x| CollectionBid {
//             collection: x.collection,
//             bidder: x.bidder,
//             price: x.price,
//             finders_fee_bps: None,
//             expires_at: env.block.time.plus_days(1_000),
//         })
//         .collect();

//     Ok(CollectionBidsResponse { bids })
// }

// pub fn bids_sorted_by_price(
//     deps: Deps,
//     env: Env,
//     collection: Collection,
//     _start_before: Option<BidOffset>,
//     limit: Option<u32>,
// ) -> StdResult<BidsResponse> {
//     let limit = limit
//         .unwrap_or(PAGINATION_DEFAULT_LIMIT)
//         .min(PAGINATION_MAX_LIMIT) as usize;

//     let bids: Vec<Bid> = COLLECTION_OFFERS
//         .load(deps.storage)?
//         .into_iter()
//         .filter(|x| x.collection == collection)
//         .take(limit)
//         .map(|x| Bid {
//             collection: x.collection,
//             bidder: x.bidder,
//             price: x.price,
//             token_id: 5,
//             finders_fee_bps: None,
//             expires_at: env.block.time.plus_days(1_000),
//         })
//         .collect();

//     Ok(BidsResponse { bids })
// }

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
