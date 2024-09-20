use cosmwasm_std::{Addr, Deps, Env, QuerierWrapper, StdResult, Uint128};

use cw721::TokensResponse;

use sg_marketplace::msg::CollectionBidResponse;

use goplend_base::{
    lending_platform::types::CollectionInfo,
    market_maker::{
        outpost::state::PAGINATION_DEFAULT_LIMIT,
        types::{CollectionOwners, OffersResponse},
    },
    utils::unwrap_field,
};

use crate::helpers::get_config;

pub fn query_collateral_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    let (_, hub) = get_config(deps)?;

    Ok(get_collection_list(&deps.querier, &hub, start_after, limit)
        .iter()
        .map(|x| CollectionInfo {
            collection_address: x.collection_address.clone(),
            token_id_list: get_token_id_list(
                &deps.querier,
                &x.collection_address,
                env.contract.address.clone(),
            ),
        })
        .collect())
}

pub fn query_collateral(
    deps: Deps,
    env: Env,
    collection_address: String,
) -> StdResult<CollectionInfo<Addr>> {
    Ok(CollectionInfo {
        collection_address: deps.api.addr_validate(&collection_address)?,
        token_id_list: get_token_id_list(&deps.querier, &collection_address, env.contract.address),
    })
}

pub fn query_offer_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<OffersResponse>> {
    let (config, hub) = get_config(deps)?;
    let marketplace = &unwrap_field(config.marketplace, "marketplace")?;
    let outpost = env.contract.address;

    get_collection_list(&deps.querier, &hub, start_after, limit)
        .iter()
        .map(|x| {
            let offer_price =
                get_offer_price(&deps.querier, marketplace, &x.collection_address, &outpost);

            Ok(OffersResponse {
                collection_address: x.collection_address.clone(),
                price_and_outpost_list: vec![(offer_price, outpost.clone())],
            })
        })
        .collect()
}

pub fn query_offer(deps: Deps, env: Env, collection_address: String) -> StdResult<Uint128> {
    let (config, _) = get_config(deps)?;

    Ok(get_offer_price(
        &deps.querier,
        unwrap_field(config.marketplace, "marketplace")?,
        collection_address,
        env.contract.address,
    ))
}

fn get_collection_list(
    querier: &QuerierWrapper,
    hub: &Addr,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Vec<CollectionOwners<Addr>> {
    querier
        .query_wasm_smart::<Vec<CollectionOwners<Addr>>>(
            hub,
            &goplend_base::market_maker::hub::msg::QueryMsg::QueryCollectionOwnersList {
                start_after,
                limit,
            },
        )
        .unwrap_or_default()
}

fn get_token_id_list(
    querier: &QuerierWrapper,
    collection: impl ToString,
    owner: impl ToString,
) -> Vec<String> {
    querier
        .query_wasm_smart::<TokensResponse>(
            collection.to_string(),
            &cw721::Cw721QueryMsg::Tokens {
                owner: owner.to_string(),
                start_after: None,
                limit: Some(PAGINATION_DEFAULT_LIMIT),
            },
        )
        .map(|x| x.tokens)
        .unwrap_or_default()
}

fn get_offer_price(
    querier: &QuerierWrapper,
    marketplace: impl ToString,
    collection: impl ToString,
    bidder: impl ToString,
) -> Uint128 {
    querier
        .query_wasm_smart::<CollectionBidResponse>(
            marketplace.to_string(),
            &sg_marketplace::msg::QueryMsg::CollectionBid {
                collection: collection.to_string(),
                bidder: bidder.to_string(),
            },
        )
        .map(|x| x.bid.map(|y| y.price).unwrap_or_default())
        .unwrap_or_default()
}
