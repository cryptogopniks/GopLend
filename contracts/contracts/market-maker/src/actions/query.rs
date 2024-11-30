use cosmwasm_std::{Addr, Deps, Env, Order, StdResult};

use cw_storage_plus::Bound;

use goplend_base::{
    lending_platform::types::CollectionInfo,
    market_maker::{
        state::{COLLATERALS, CONFIG, OFFERS, OWNERS, UNDISTRIBUTED_LIQUIDITY},
        types::{
            CollateralListResponseItem, CollectionOwner, Config, LiquidityInfo, OffersResponse,
        },
    },
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_offer_prices_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_after: Option<String>,
) -> StdResult<Vec<OffersResponse>> {
    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(OFFERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .map(|x| {
            let (collection_address, price_list) = x.unwrap();

            OffersResponse {
                collection_address,
                price_list,
            }
        })
        .collect())
}

pub fn query_offer_prices(deps: Deps, _env: Env, collection: String) -> StdResult<OffersResponse> {
    let collection_address = deps.api.addr_validate(&collection)?;
    let price_list = OFFERS
        .load(deps.storage, &collection_address)
        .unwrap_or_default();

    Ok(OffersResponse {
        collection_address,
        price_list,
    })
}

pub fn query_collection_owner_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_after: Option<String>,
) -> StdResult<Vec<CollectionOwner<Addr>>> {
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
        .take(amount as usize)
        .map(|x| {
            let (collection_address, owner) = x.unwrap();

            Ok(CollectionOwner {
                collection_address,
                owner,
            })
        })
        .collect()
}

pub fn query_collection_owner(
    deps: Deps,
    _env: Env,
    collection: String,
) -> StdResult<CollectionOwner<Addr>> {
    let collection_address = deps.api.addr_validate(&collection)?;
    let owner = OWNERS.load(deps.storage, &collection_address)?;

    Ok(CollectionOwner {
        collection_address,
        owner,
    })
}

pub fn query_liquidity_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_after: Option<String>,
) -> StdResult<Vec<LiquidityInfo>> {
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
        .take(amount as usize)
        .map(|x| {
            let (collection_address, amount_undistributed) = x.unwrap();
            let price_list = OFFERS
                .load(deps.storage, &collection_address)
                .unwrap_or_default();

            Ok(LiquidityInfo {
                collection_address,
                amount_undistributed,
                price_list,
            })
        })
        .collect()
}

pub fn query_liquidity(deps: Deps, _env: Env, collection: String) -> StdResult<LiquidityInfo> {
    let collection_address = deps.api.addr_validate(&collection)?;
    let amount_undistributed = UNDISTRIBUTED_LIQUIDITY
        .load(deps.storage, &collection_address)
        .unwrap_or_default();
    let price_list = OFFERS
        .load(deps.storage, &collection_address)
        .unwrap_or_default();

    Ok(LiquidityInfo {
        collection_address,
        amount_undistributed,
        price_list,
    })
}

pub fn query_collateral_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_after: Option<String>,
) -> StdResult<Vec<CollateralListResponseItem>> {
    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(COLLATERALS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .map(|x| {
            let (collection_owner, collection_list) = x.unwrap();

            CollateralListResponseItem {
                collection_owner,
                collection_list,
            }
        })
        .collect())
}

pub fn query_collateral(
    deps: Deps,
    _env: Env,
    collection_owner: String,
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    let collection_owner = deps.api.addr_validate(&collection_owner)?;

    Ok(COLLATERALS
        .load(deps.storage, &collection_owner)
        .unwrap_or_default())
}
