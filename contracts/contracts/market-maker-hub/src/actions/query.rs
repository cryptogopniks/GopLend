use cosmwasm_std::{Addr, Deps, Env, Order, QuerierWrapper, StdResult, Uint128};

use cw_storage_plus::Bound;

use goplend_base::{
    lending_platform::types::CollectionInfo,
    market_maker::{
        hub::state::{
            CONFIG, OUTPOSTS, OWNERS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT,
            UNDISTRIBUTED_LIQUIDITY,
        },
        types::{CollateralResponse, CollectionOwners, HubConfig, LiquidityInfo},
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
        .take(limit)
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

    UNDISTRIBUTED_LIQUIDITY
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (collection_address, amount_undistributed) = x.unwrap();
            let price_and_outpost_list =
                get_price_and_outpost_list(&deps.querier, &collection_address, &outpost_list)?;

            Ok(LiquidityInfo {
                collection_address,
                amount_undistributed,
                price_and_outpost_list,
            })
        })
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_liquidity(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<LiquidityInfo> {
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let amount_undistributed = UNDISTRIBUTED_LIQUIDITY.load(deps.storage, &collection_address)?;
    let outpost_list = OUTPOSTS.load(deps.storage)?;
    let price_and_outpost_list =
        get_price_and_outpost_list(&deps.querier, &collection_address, &outpost_list)?;

    Ok(LiquidityInfo {
        collection_address,
        amount_undistributed,
        price_and_outpost_list,
    })
}

pub fn query_collateral_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollateralResponse>> {
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

    let collection_list = OWNERS
        .keys(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<Addr>>>()?;

    Ok(outpost_list
        .iter()
        .flat_map(|outpost| {
            collection_list
                .iter()
                .map(|collection| {
                    Ok(CollateralResponse {
                        outpost: outpost.to_owned(),
                        collection_info: get_collection_info(&deps.querier, collection, outpost)?,
                    })
                })
                .collect::<StdResult<Vec<CollateralResponse>>>()
                .unwrap_or_default()
        })
        .collect())
}

pub fn query_collateral(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<Vec<CollateralResponse>> {
    let collection_address = deps.api.addr_validate(&collection_address)?;

    OUTPOSTS
        .load(deps.storage)?
        .iter()
        .map(|outpost| {
            Ok(CollateralResponse {
                outpost: outpost.to_owned(),
                collection_info: get_collection_info(&deps.querier, &collection_address, outpost)?,
            })
        })
        .collect()
}

pub fn get_price_and_outpost_list(
    querier: &QuerierWrapper,
    collection_address: &Addr,
    outpost_list: &[Addr],
) -> StdResult<Vec<(Uint128, Addr)>> {
    outpost_list
        .iter()
        .map(|outpost| {
            let price = querier.query_wasm_smart::<Uint128>(
                outpost,
                &goplend_base::market_maker::outpost::msg::QueryMsg::QueryOffer {
                    collection_address: collection_address.to_string(),
                },
            )?;

            Ok((price, outpost.to_owned()))
        })
        .collect::<StdResult<Vec<_>>>()
}

fn get_collection_info(
    querier: &QuerierWrapper,
    collection_address: &Addr,
    outpost: &Addr,
) -> StdResult<CollectionInfo<Addr>> {
    querier.query_wasm_smart(
        outpost,
        &goplend_base::market_maker::outpost::msg::QueryMsg::QueryCollateral {
            collection_address: collection_address.to_string(),
        },
    )
}
