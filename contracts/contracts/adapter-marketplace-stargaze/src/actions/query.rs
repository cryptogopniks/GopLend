use cosmwasm_std::{Decimal, Deps, Env, StdResult};

use sg_marketplace::{
    msg::{CollectionBidsResponse, QueryMsg as MarketplaceQueryMsg},
    state::CollectionBid,
};

use goplend_base::{
    adapters::marketplace::stargaze::{
        state::{CONFIG, PAGINATION_DEFAULT_LIMIT},
        types::Config,
    },
    lending_platform::{
        msg::{
            QueryCollectionsResponseItem, QueryLiquidationBidsByCollectionAddressListResponseItem,
            QueryMsg as LendingPlatformQueryMsg,
        },
        types::{Collateral, LiquidationBid},
    },
    utils::unwrap_field,
};

// TODO: add infinity pool offers

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_liquidation_bids_by_collection_address_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
    let block_time = env.block.time.seconds();
    let Config {
        lending_platform,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    let lending_platform = &unwrap_field(lending_platform, "lending_platform")?;
    let marketplace = &unwrap_field(marketplace, "marketplace")?;

    let collection_list: Vec<QueryCollectionsResponseItem> = deps.querier.query_wasm_smart(
        lending_platform,
        &LendingPlatformQueryMsg::QueryCollectionList { start_after, limit },
    )?;

    let mut total_liquidation_bids: Vec<QueryLiquidationBidsByCollectionAddressListResponseItem> =
        vec![];

    for QueryCollectionsResponseItem { address, .. } in collection_list {
        let CollectionBidsResponse { bids } = deps.querier.query_wasm_smart(
            marketplace,
            // &MarketplaceQueryMsg::CollectionOffersByCollection {
            //     collection: address.to_string(),
            //     query_options: None,
            // },
            &MarketplaceQueryMsg::CollectionBidsSortedByPrice {
                collection: address.to_string(),
                start_after: None,
                limit: Some(PAGINATION_DEFAULT_LIMIT),
            },
        )?;

        let liquidation_bids = bids
            .into_iter()
            .map(
                |CollectionBid {
                     collection,
                     price,
                     bidder,
                     ..
                 }|
                 -> StdResult<LiquidationBid> {
                    let collateral: Vec<Collateral> = deps.querier.query_wasm_smart(
                        lending_platform,
                        &LendingPlatformQueryMsg::QueryCollateral {
                            collection_address: collection.to_string(),
                        },
                    )?;

                    // TODO: check if it's fine to pass all tokens including tokens owned by liquidators
                    let token_id_list = collateral
                        .into_iter()
                        .map(|x| x.token_id_list)
                        .flatten()
                        .collect();

                    Ok(LiquidationBid {
                        creation_date: block_time,
                        liquidator: bidder,
                        token_id_list,
                        amount: price,
                        discount: Decimal::zero(), // TODO: order_info.finders_fee_bps?
                        is_offer: true,
                    })
                },
            )
            .collect::<StdResult<Vec<LiquidationBid>>>()?;

        total_liquidation_bids.push(QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: address,
            liquidation_bids,
        });
    }

    Ok(total_liquidation_bids)
}

pub fn query_liquidation_bids_by_collection_address(
    deps: Deps,
    env: Env,
    address: String,
) -> StdResult<Vec<LiquidationBid>> {
    let block_time = env.block.time.seconds();
    let collection = &deps.api.addr_validate(&address)?;
    let Config {
        lending_platform,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    let lending_platform = &unwrap_field(lending_platform, "lending_platform")?;
    let marketplace = &unwrap_field(marketplace, "marketplace")?;
    // let infinity_pool = &unwrap_field(infinity_pool, "infinity_pool")?;

    // TODO: check if it will return token offers as well
    let CollectionBidsResponse { bids } = deps.querier.query_wasm_smart(
        marketplace,
        // &MarketplaceQueryMsg::CollectionOffersByCollection {
        //     collection: address.to_string(),
        //     query_options: None,
        // },
        &MarketplaceQueryMsg::CollectionBidsSortedByPrice {
            collection: address.to_string(),
            start_after: None,
            limit: Some(PAGINATION_DEFAULT_LIMIT),
        },
    )?;

    let liquidation_bids = bids
        .into_iter()
        .map(
            |CollectionBid { price, bidder, .. }| -> StdResult<LiquidationBid> {
                let collateral: Vec<Collateral> = deps.querier.query_wasm_smart(
                    lending_platform,
                    &LendingPlatformQueryMsg::QueryCollateral {
                        collection_address: collection.to_string(),
                    },
                )?;

                // TODO: check if it's fine to pass all tokens including tokens owned by liquidators
                let token_id_list = collateral
                    .into_iter()
                    .map(|x| x.token_id_list)
                    .flatten()
                    .collect();

                Ok(LiquidationBid {
                    creation_date: block_time,
                    liquidator: bidder,
                    token_id_list,
                    amount: price,
                    discount: Decimal::zero(), // TODO: order_info.finders_fee_bps?
                    is_offer: true,
                })
            },
        )
        .collect::<StdResult<Vec<LiquidationBid>>>()?;

    Ok(liquidation_bids)
}

// pub fn query_liquidation_bids_by_liquidator_address_list(
//     deps: Deps,
//     env: Env,
//     start_after: Option<String>,
//     limit: Option<u32>,
//     // liquidator_address_list: Vec<String>
// ) -> StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>> {
//     let block_time = env.block.time.seconds();
//     let Config {
//         lending_platform,
//         marketplace,
//         ..
//     } = CONFIG.load(deps.storage)?;
//     let lending_platform = &unwrap_field(lending_platform, "lending_platform")?;
//     let marketplace = &unwrap_field(marketplace, "marketplace")?;
//     // let infinity_pool = &unwrap_field(infinity_pool, "infinity_pool")?;

//     let collection_offers: Vec<CollectionOffer> = deps.querier.query_wasm_smart(
//         marketplace,
//         &MarketplaceQueryMsg::CollectionOffersByCreator {
//             creator: address,
//             query_options: None,
//         },
//     )?;

//     let liquidation_bids = collection_offers
//         .into_iter()
//         .map(
//             |CollectionOffer {
//                  collection,
//                  order_info,
//              }| {
//                 let collateral: Vec<Collateral> = deps.querier.query_wasm_smart(
//                     lending_platform,
//                     &LendingPlatformQueryMsg::QueryCollateral {
//                         collection_address: collection.to_string(),
//                     },
//                 )?;

//                 let token_id_list = collateral
//                     .into_iter()
//                     .map(|x| x.token_id_list)
//                     .flatten()
//                     .collect();

//                 Ok(QueryLiquidationBidsByCollectionAddressListResponseItem {
//                     collection_address: collection,
//                     liquidation_bids: vec![LiquidationBid {
//                         creation_date: block_time,
//                         liquidator: order_info.creator,
//                         token_id_list, // all tokens
//                         amount: order_info.price.amount,
//                         discount: Decimal::zero(), // TODO: order_info.finders_fee_bps?
//                         is_offer: true,
//                     }],
//                 })
//             },
//         )
//         .collect::<StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>>()?;

//     Ok(liquidation_bids)

//     Ok(vec![])
// }

pub fn query_liquidation_bids_by_liquidator_address(
    deps: Deps,
    env: Env,
    address: String,
) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
    let block_time = env.block.time.seconds();
    let Config {
        lending_platform,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    let lending_platform = &unwrap_field(lending_platform, "lending_platform")?;
    let marketplace = &unwrap_field(marketplace, "marketplace")?;
    // let infinity_pool = &unwrap_field(infinity_pool, "infinity_pool")?;

    let CollectionBidsResponse { bids } = deps.querier.query_wasm_smart(
        marketplace,
        // &MarketplaceQueryMsg::CollectionOffersByCollection {
        //     collection: address.to_string(),
        //     query_options: None,
        // },
        &MarketplaceQueryMsg::CollectionBidsByBidder {
            bidder: address,
            start_after: None,
            limit: Some(PAGINATION_DEFAULT_LIMIT),
        },
    )?;

    let liquidation_bids = bids
        .into_iter()
        .map(
            |CollectionBid {
                 collection,
                 price,
                 bidder,
                 ..
             }| {
                let collateral: Vec<Collateral> = deps.querier.query_wasm_smart(
                    lending_platform,
                    &LendingPlatformQueryMsg::QueryCollateral {
                        collection_address: collection.to_string(),
                    },
                )?;

                // TODO: check if it's fine to pass all tokens including tokens owned by liquidators
                let token_id_list = collateral
                    .into_iter()
                    .map(|x| x.token_id_list)
                    .flatten()
                    .collect();

                Ok(QueryLiquidationBidsByCollectionAddressListResponseItem {
                    collection_address: collection,
                    liquidation_bids: vec![LiquidationBid {
                        creation_date: block_time,
                        liquidator: bidder,
                        token_id_list,
                        amount: price,
                        discount: Decimal::zero(), // TODO: order_info.finders_fee_bps?
                        is_offer: true,
                    }],
                })
            },
        )
        .collect::<StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>>()?;

    Ok(liquidation_bids)
}
