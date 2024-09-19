use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};

use cw721::TokensResponse;

use sg_marketplace::msg::CollectionBidResponse;

use goplend_base::{
    error::ContractError,
    lending_platform::types::CollectionInfo,
    market_maker::{
        outpost::state::{PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT},
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
    let collection_list = deps
        .querier
        .query_wasm_smart::<Vec<CollectionOwners<Addr>>>(
            &hub,
            &goplend_base::market_maker::hub::msg::QueryMsg::QueryCollectionOwnersList {
                start_after,
                limit,
            },
        )?;

    collection_list
        .iter()
        .map(|x| {
            let TokensResponse { tokens } = deps.querier.query_wasm_smart(
                &x.collection_address,
                &cw721::Cw721QueryMsg::Tokens {
                    owner: env.contract.address.to_string(),
                    start_after: None,
                    limit: Some(PAGINATION_MAX_LIMIT),
                },
            )?;

            Ok(CollectionInfo {
                collection_address: x.collection_address.clone(),
                token_id_list: tokens,
            })
        })
        .collect()
}

pub fn query_collateral(
    deps: Deps,
    env: Env,
    collection_address: String,
) -> StdResult<CollectionInfo<Addr>> {
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let TokensResponse { tokens } = deps.querier.query_wasm_smart(
        &collection_address,
        &cw721::Cw721QueryMsg::Tokens {
            owner: env.contract.address.to_string(),
            start_after: None,
            limit: Some(PAGINATION_DEFAULT_LIMIT),
        },
    )?;

    Ok(CollectionInfo {
        collection_address,
        token_id_list: tokens,
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
    let collection_list = deps
        .querier
        .query_wasm_smart::<Vec<CollectionOwners<Addr>>>(
            &hub,
            &goplend_base::market_maker::hub::msg::QueryMsg::QueryCollectionOwnersList {
                start_after,
                limit,
            },
        )?;

    collection_list
        .iter()
        .map(|x| {
            let offer_price = deps
                .querier
                .query_wasm_smart::<CollectionBidResponse>(
                    marketplace,
                    &sg_marketplace::msg::QueryMsg::CollectionBid {
                        collection: x.collection_address.to_string(),
                        bidder: outpost.to_string(),
                    },
                )?
                .bid
                .ok_or(ContractError::OfferIsNotFound)?
                .price;

            Ok(OffersResponse {
                collection_address: x.collection_address.clone(),
                price_and_outpost_list: vec![(offer_price, outpost.clone())],
            })
        })
        .collect()
}

pub fn query_offer(deps: Deps, env: Env, collection_address: String) -> StdResult<Uint128> {
    let (config, _) = get_config(deps)?;
    let offer_price = deps
        .querier
        .query_wasm_smart::<CollectionBidResponse>(
            unwrap_field(config.marketplace, "marketplace")?,
            &sg_marketplace::msg::QueryMsg::CollectionBid {
                collection: collection_address,
                bidder: env.contract.address.to_string(),
            },
        )?
        .bid
        .ok_or(ContractError::OfferIsNotFound)?
        .price;

    Ok(offer_price)
}
