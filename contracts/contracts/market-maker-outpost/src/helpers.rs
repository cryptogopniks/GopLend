use cosmwasm_std::{Addr, Deps, QuerierWrapper, StdResult};

use sg_index_query::QueryOptions;
use stargaze_marketplace_v2::orders::CollectionBid;

use goplend_base::{
    error::ContractError,
    market_maker::{
        outpost::state::CONFIG,
        types::{HubConfig, OutpostConfig},
    },
};

/// returns (hub_config, hub_address)
pub fn get_config(deps: Deps) -> StdResult<(HubConfig, Addr)> {
    let OutpostConfig { hub } = CONFIG.load(deps.storage)?;
    let config = deps.querier.query_wasm_smart::<HubConfig>(
        &hub,
        &goplend_base::market_maker::hub::msg::QueryMsg::QueryConfig {},
    )?;

    Ok((config, hub))
}

pub fn get_outpost_offer(
    querier: &QuerierWrapper,
    marketplace: impl ToString,
    collection: impl ToString,
    bidder: impl ToString,
) -> StdResult<CollectionBid> {
    let collection_bids: Vec<CollectionBid> = querier.query_wasm_smart(
        marketplace.to_string(),
        &stargaze_marketplace_v2::msg::QueryMsg::CollectionBidsByCreatorCollection {
            creator: bidder.to_string(),
            collection: collection.to_string(),
            query_options: Some(QueryOptions {
                descending: Some(true),
                limit: Some(1),
                min: None,
                max: None,
            }),
        },
    )?;

    Ok(collection_bids
        .first()
        .ok_or(ContractError::OfferIsNotFound)?
        .to_owned())
}
