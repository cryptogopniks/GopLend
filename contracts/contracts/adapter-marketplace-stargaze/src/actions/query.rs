use cosmwasm_std::{Addr, Decimal, Deps, Env, StdResult, Uint128};

use sg_index_query::QueryOptions;
use stargaze_marketplace_v2::orders::{Ask, Bid, CollectionBid};

use goplend_base::{
    adapters::marketplace::stargaze::{
        msg::QueryCollectionOffersResponse, state::CONFIG, types::Config,
    },
    oracle::types::PriceRatioItem,
    utils::unwrap_field,
};

use crate::helpers::{get_discount_base, get_main_denom};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_floor_prices(
    deps: Deps,
    _env: Env,
    collection_address_list: Vec<String>,
) -> StdResult<Vec<(Addr, Uint128)>> {
    let Config {
        marketplace,
        oracle,
        ..
    } = CONFIG.load(deps.storage)?;
    let main_denom = get_main_denom(&deps.querier, &oracle)?;

    collection_address_list
        .iter()
        .map(|collection| {
            let collection_address = deps.api.addr_validate(collection)?;
            let asks: Vec<Ask> = deps.querier.query_wasm_smart(
                marketplace.clone(),
                &stargaze_marketplace_v2::msg::QueryMsg::AsksByCollectionDenom {
                    collection: collection.to_owned(),
                    denom: main_denom.clone(),
                    query_options: Some(QueryOptions {
                        // TODO: will it give FP?
                        descending: Some(false),
                        limit: Some(1),
                        min: None,
                        max: None,
                    }),
                },
            )?;
            let floor_price = asks
                .first()
                .map(|y| y.details.price.amount)
                .unwrap_or_default();

            Ok((collection_address, floor_price))
        })
        .collect()
}

pub fn query_token_offers(
    deps: Deps,
    _env: Env,
    collection_address: String,
    token_list: Vec<String>,
    limit: u32, // max 100
) -> StdResult<Vec<(String, Vec<Bid>)>> {
    let Config {
        marketplace,
        oracle,
        ..
    } = CONFIG.load(deps.storage)?;
    let main_denom = get_main_denom(&deps.querier, &oracle)?;

    token_list
        .into_iter()
        .map(|token_id| -> StdResult<(String, Vec<Bid>)> {
            let bids: Vec<Bid> = deps.querier.query_wasm_smart(
                marketplace.clone(),
                &stargaze_marketplace_v2::msg::QueryMsg::BidsByTokenPrice {
                    collection: collection_address.clone(),
                    token_id: token_id.clone(),
                    denom: main_denom.clone(),
                    query_options: Some(QueryOptions {
                        descending: Some(true),
                        limit: Some(limit),
                        min: None,
                        max: None,
                    }),
                },
            )?;

            Ok((token_id, bids))
        })
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_collection_offers(
    deps: Deps,
    _env: Env,
    collection_address: String,
    limit: u32,
) -> StdResult<QueryCollectionOffersResponse> {
    let Config {
        oracle,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    let main_denom = get_main_denom(&deps.querier, &oracle)?;

    // this query is used to get collection main prices for oracle
    // then price_ratio_stable can be optional
    let price_ratio_item = deps
        .querier
        .query_wasm_smart::<PriceRatioItem>(
            &oracle,
            &goplend_base::oracle::msg::QueryMsg::QueryPriceRatio {},
        )
        .unwrap_or(PriceRatioItem {
            value: Decimal::zero(),
            update_date: 0,
            is_outdated: true,
        });

    let price_ratio_stable = if price_ratio_item.is_outdated {
        None
    } else {
        Some(price_ratio_item.value)
    };

    let discount_base = get_discount_base(&deps.querier, &marketplace, &collection_address)?;

    let collection_bids: Vec<CollectionBid> = deps.querier.query_wasm_smart(
        marketplace,
        &stargaze_marketplace_v2::msg::QueryMsg::CollectionBidsByPrice {
            collection: collection_address.clone(),
            denom: main_denom,
            query_options: Some(QueryOptions {
                descending: Some(true),
                limit: Some(limit),
                min: None,
                max: None,
            }),
        },
    )?;

    Ok(QueryCollectionOffersResponse {
        collection_bids,
        discount_base,
        price_ratio_stable,
    })
}
