use cosmwasm_std::{Addr, Deps, Env, Order, QuerierWrapper, StdResult, Uint128};

use cw721::TokensResponse;
use cw_storage_plus::Bound;

use goplend_base::{
    lending_platform::types::CollectionInfo,
    market_maker::{
        state::{CONFIG, OFFERS, OWNERS, UNDISTRIBUTED_LIQUIDITY},
        types::{CollectionOwner, Config, LiquidityInfo},
    },
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
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
        .collect::<StdResult<Vec<_>>>()
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
            let price_list = OFFERS.load(deps.storage, &collection_address)?;

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
    let amount_undistributed = UNDISTRIBUTED_LIQUIDITY.load(deps.storage, &collection_address)?;
    let price_list = OFFERS.load(deps.storage, &collection_address)?;

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
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    // let binding;
    // let start_bound = match start_after {
    //     Some(addr) => {
    //         binding = deps.api.addr_validate(&addr)?;
    //         Some(Bound::exclusive(&binding))
    //     }
    //     None => None,
    // };

    //  Ok(OFFERS
    //     .keys(deps.storage, start_bound, None, Order::Ascending)
    //     .take(amount as usize)
    //     .map(|x| {

    //     })
    //     .collect())

    // Ok(collection_list
    //             .iter()
    //             .map(|collection| {
    //                 Ok(CollectionInfo<Addr> {
    //                     outpost: outpost.to_owned(),
    //                     collection_info: get_collection_info(&deps.querier, collection, outpost)?,
    //                 })
    //             })
    //     .collect())

    unimplemented!()
}

pub fn query_collateral(
    deps: Deps,
    _env: Env,
    collection: String,
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    let collection_address = deps.api.addr_validate(&collection)?;

    // OUTPOSTS
    //     .load(deps.storage)?
    //     .iter()
    //     .map(|outpost| {
    //         Ok(CollectionInfo<Addr> {
    //             outpost: outpost.to_owned(),
    //             collection_info: get_collection_info(&deps.querier, &collection_address, outpost)?,
    //         })
    //     })
    //     .collect()

    unimplemented!()
}

pub fn get_price_and_outpost_list(
    querier: &QuerierWrapper,
    collection_address: &Addr,
    outpost_list: &[Addr],
) -> StdResult<Vec<(Uint128, Addr)>> {
    // outpost_list
    //     .iter()
    //     .map(|outpost| {
    //         let price = querier.query_wasm_smart::<Uint128>(
    //             outpost,
    //             &goplend_base::market_maker::outpost::msg::QueryMsg::QueryOffer {
    //                 collection_address: collection_address.to_string(),
    //             },
    //         )?;

    //         Ok((price, outpost.to_owned()))
    //     })
    //     .collect::<StdResult<Vec<_>>>()

    unimplemented!()
}

fn get_collection_info(
    querier: &QuerierWrapper,
    collection_address: &Addr,
    outpost: &Addr,
) -> StdResult<CollectionInfo<Addr>> {
    // querier.query_wasm_smart(
    //     outpost,
    //     &goplend_base::market_maker::outpost::msg::QueryMsg::QueryCollateral {
    //         collection_address: collection_address.to_string(),
    //     },
    // )

    unimplemented!()
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
                limit: Some(50), // TODO: replace
            },
        )
        .map(|x| x.tokens)
        .unwrap_or_default()
}
