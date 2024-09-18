use cosmwasm_std::{Addr, Deps, Env, Order, StdResult, Uint128};

use cw_storage_plus::Bound;

use goplend_base::{
    lending_platform::types::CollectionInfo,
    market_maker::{
        hub::state::{
            CONFIG, LIQUIDITY, OUTPOSTS, OWNERS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT,
        },
        types::{CollectionOwners, HubConfig, LiquidityInfo, OffersResponse},
    },
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<HubConfig> {
    CONFIG.load(deps.storage)
}

pub fn query_outposts(deps: Deps, _env: Env) -> StdResult<Vec<Addr>> {
    OUTPOSTS.load(deps.storage)
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
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    let outpost_list = OUTPOSTS.load(deps.storage)?;
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

            let token_id_list = outpost_list
                .iter()
                .flat_map(|x| {
                    deps.querier
                        .query_wasm_smart::<CollectionInfo<Addr>>(
                            x,
                            &goplend_base::market_maker::outpost::msg::QueryMsg::QueryCollateral {
                                collection_address: collection_address.to_string(),
                            },
                        )
                        .map(|y| y.token_id_list)
                        .unwrap_or_default()
                })
                .collect();

            Ok(CollectionInfo {
                collection_address,
                token_id_list,
            })
        })
        .collect()
}

pub fn query_collateral(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<CollectionInfo<Addr>> {
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let outpost_list = OUTPOSTS.load(deps.storage)?;
    let token_id_list = outpost_list
        .iter()
        .flat_map(|x| {
            deps.querier
                .query_wasm_smart::<CollectionInfo<Addr>>(
                    x,
                    &goplend_base::market_maker::outpost::msg::QueryMsg::QueryCollateral {
                        collection_address: collection_address.to_string(),
                    },
                )
                .map(|y| y.token_id_list)
                .unwrap_or_default()
        })
        .collect();

    Ok(CollectionInfo {
        collection_address,
        token_id_list,
    })
}

pub fn query_offers_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<OffersResponse>> {
    let outpost_list = OUTPOSTS.load(deps.storage)?;
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
            let price_list = outpost_list
                .iter()
                .map(|x| {
                    deps.querier.query_wasm_smart::<Uint128>(
                        x,
                        &goplend_base::market_maker::outpost::msg::QueryMsg::QueryOffer {
                            collection_address: collection_address.to_string(),
                        },
                    )
                })
                .collect::<StdResult<Vec<_>>>()?;

            Ok(OffersResponse {
                collection_address,
                price_list,
            })
        })
        .collect()
}

pub fn query_offers(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<OffersResponse> {
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let outpost_list = OUTPOSTS.load(deps.storage)?;
    let price_list = outpost_list
        .iter()
        .map(|x| {
            deps.querier.query_wasm_smart::<Uint128>(
                x,
                &goplend_base::market_maker::outpost::msg::QueryMsg::QueryOffer {
                    collection_address: collection_address.to_string(),
                },
            )
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(OffersResponse {
        collection_address,
        price_list,
    })
}
