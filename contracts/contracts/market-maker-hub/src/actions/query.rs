use cosmwasm_std::{Addr, Deps, Env, Order, StdResult, Uint128};

use cw721::TokensResponse;
use cw_storage_plus::Bound;

use sg_marketplace::msg::{CollectionBidsResponse, QueryMsg as MarketplaceQueryMsg};

use goplend_base::{
    lending_platform::types::CollectionInfo,
    market_maker::{
        hub::state::{CONFIG, LIQUIDITY, OWNERS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT},
        types::{CollectionOwners, HubConfig, LiquidityInfo, OffersResponse},
    },
    utils::unwrap_field,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<HubConfig> {
    CONFIG.load(deps.storage)
}

pub fn query_collection_owners_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollectionOwners<Addr>>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    OWNERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit as usize)
        .map(|x| {
            let (collection_address, owner_list) = x.unwrap();

            Ok(CollectionOwners {
                collection_address,
                owner_list,
            })
        })
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_collection_owners(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<CollectionOwners<Addr>> {
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let owner_list = OWNERS.load(deps.storage, &collection_address)?;

    Ok(CollectionOwners {
        collection_address,
        owner_list,
    })
}

pub fn query_liquidity_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<LiquidityInfo>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    LIQUIDITY
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit as usize)
        .map(|x| {
            let (_, liquidity_info) = x.unwrap();
            Ok(liquidity_info)
        })
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_liquidity(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<LiquidityInfo> {
    LIQUIDITY.load(deps.storage, &deps.api.addr_validate(&collection_address)?)
}

pub fn query_collateral_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    let market_maker = env.contract.address;
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    OWNERS
        .keys(deps.storage, start_bound, None, Order::Ascending)
        .take(limit as usize)
        .map(|x| {
            let collection_address = x.unwrap();

            let TokensResponse { tokens } = deps.querier.query_wasm_smart(
                &collection_address,
                &cw721::Cw721QueryMsg::Tokens {
                    owner: market_maker.to_string(),
                    start_after: None,
                    limit: Some(PAGINATION_MAX_LIMIT),
                },
            )?;

            Ok(CollectionInfo {
                collection_address,
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
    let market_maker = env.contract.address;
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let TokensResponse { tokens } = deps.querier.query_wasm_smart(
        &collection_address,
        &cw721::Cw721QueryMsg::Tokens {
            owner: market_maker.to_string(),
            start_after: None,
            limit: Some(PAGINATION_MAX_LIMIT),
        },
    )?;

    Ok(CollectionInfo {
        collection_address,
        token_id_list: tokens,
    })
}

pub fn query_offers_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<OffersResponse>> {
    let market_maker = env.contract.address;
    let config = CONFIG.load(deps.storage)?;
    let marketplace = &unwrap_field(config.marketplace, "marketplace")?;

    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    OWNERS
        .keys(deps.storage, start_bound, None, Order::Ascending)
        .take(limit as usize)
        .map(|x| {
            let collection_address = x.unwrap();

            let CollectionBidsResponse {
                bids: collection_bids,
            } = deps.querier.query_wasm_smart(
                marketplace,
                &MarketplaceQueryMsg::ReverseCollectionBidsSortedByPrice {
                    collection: collection_address.to_string(),
                    start_before: None,
                    limit: Some(PAGINATION_MAX_LIMIT),
                },
            )?;

            let price_list: Vec<Uint128> = collection_bids.iter().fold(vec![], |mut acc, cur| {
                if cur.bidder == market_maker {
                    acc.push(cur.price);
                }

                acc
            });

            Ok(OffersResponse {
                collection_address,
                price_list,
            })
        })
        .collect()
}

pub fn query_offers(deps: Deps, env: Env, collection_address: String) -> StdResult<OffersResponse> {
    let market_maker = env.contract.address;
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let config = CONFIG.load(deps.storage)?;
    let marketplace = unwrap_field(config.marketplace, "marketplace")?;
    let CollectionBidsResponse {
        bids: collection_bids,
    } = deps.querier.query_wasm_smart(
        marketplace,
        &MarketplaceQueryMsg::ReverseCollectionBidsSortedByPrice {
            collection: collection_address.to_string(),
            start_before: None,
            limit: Some(PAGINATION_MAX_LIMIT),
        },
    )?;

    let price_list: Vec<Uint128> = collection_bids.iter().fold(vec![], |mut acc, cur| {
        if cur.bidder == market_maker {
            acc.push(cur.price);
        }

        acc
    });

    Ok(OffersResponse {
        collection_address,
        price_list,
    })
}
