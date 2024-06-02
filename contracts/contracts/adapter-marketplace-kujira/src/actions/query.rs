use cosmwasm_std::{Deps, Env, StdResult};

// use cw_storage_plus::Bound;
use goplend_base::adapters::marketplace::kujira::{
    state::{CONFIG, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT},
    types::Config,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_liquidation_bids_by_collection_address_list(
    _deps: Deps,
    _env: Env,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>{
    let _limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    // let binding;
    // let start_bound = match start_after {
    //     Some(addr) => {
    //         binding = deps.api.addr_validate(&addr)?;
    //         Some(Bound::exclusive(&binding))
    //     }
    //     None => None,
    // };

    // Ok(LIQUIDATION_BIDS
    //     .range(deps.storage, start_bound, None, Order::Ascending)
    //     .take(limit)
    //     .map(|x| {
    //         let (address, liquidation_bids) = x.unwrap();
    //         goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem {
    //             collection_address: address,
    //             liquidation_bids,
    //         }
    //     })
    //     .collect())

    Ok(vec![])
}

pub fn query_liquidation_bids_by_collection_address(
    _deps: Deps,
    _env: Env,
    _address: String,
) -> StdResult<Vec<goplend_base::lending_platform::types::LiquidationBid>> {
    // Ok(LIQUIDATION_BIDS
    //     .load(deps.storage, &deps.api.addr_validate(&address)?)
    //     .unwrap_or_default())

    Ok(vec![])
}

pub fn query_liquidation_bids_by_liquidator_address_list(
    _deps: Deps,
    _env: Env,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByLiquidatorAddressListResponseItem>>{
    let _limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    // let binding;
    // let start_bound = match start_after {
    //     Some(addr) => {
    //         binding = deps.api.addr_validate(&addr)?;
    //         Some(Bound::exclusive(&binding))
    //     }
    //     None => None,
    // };

    // LIQUIDATORS
    //     .range(deps.storage, start_bound, None, Order::Ascending)
    //     .take(limit)
    //     .map(|x| {
    //         let (
    //             liquidator_address,
    //             goplend_base::lending_platform::types::Liquidator {
    //                 collection_addresses,
    //             },
    //         ) = x.unwrap();

    //         let collection_and_liquidation_bids_list = collection_addresses
    //             .into_iter()
    //             .map(|collection_address| {
    //                 let liquidation_bids =
    //                     LIQUIDATION_BIDS.load(deps.storage, &collection_address)?;

    //                 Ok(goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem {
    //                     collection_address,
    //                     liquidation_bids,
    //                 })
    //             })
    //             .collect::<StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>>(
    //             )?;

    //         Ok(goplend_base::lending_platform::msg::QueryLiquidationBidsByLiquidatorAddressListResponseItem {
    //             liquidator_address,
    //             collection_and_liquidation_bids_list,
    //         })
    //     })
    //     .collect::<StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByLiquidatorAddressListResponseItem>>>()

    Ok(vec![])
}

pub fn query_liquidation_bids_by_liquidator_address(
    _deps: Deps,
    _env: Env,
    _address: String,
) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>{
    // let liquidator = LIQUIDATORS
    //     .load(deps.storage, &deps.api.addr_validate(&address)?)
    //     .unwrap_or_default();

    // liquidator
    //     .collection_addresses
    //     .into_iter()
    //     .map(|collection_address| {
    //         let liquidation_bids = LIQUIDATION_BIDS.load(deps.storage, &collection_address)?;

    //         Ok(goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem {
    //             collection_address,
    //             liquidation_bids,
    //         })
    //     })
    //     .collect::<StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>>()

    Ok(vec![])
}
