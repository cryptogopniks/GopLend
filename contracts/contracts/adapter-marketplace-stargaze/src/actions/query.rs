use cosmwasm_std::{Decimal, Deps, Env, StdResult};

use sg_marketplace::{
    msg::{BidsResponse, CollectionBidsResponse, QueryMsg as MarketplaceQueryMsg},
    state::{Bid, CollectionBid},
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
        types::{BidType, Collateral, LiquidationBid},
    },
    utils::unwrap_field,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_offers_by_collection_address_list(
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

    let mut total_offers: Vec<QueryLiquidationBidsByCollectionAddressListResponseItem> = vec![];

    for QueryCollectionsResponseItem {
        address: collection_address,
        ..
    } in collection_list
    {
        // TODO: check if it's sorted in proper order

        // try to query amount of token offers limited by PAGINATION_DEFAULT_LIMIT
        // sorted by price in descending order
        // it's important to query token offers first to get better offers
        let BidsResponse { bids } = deps.querier.query_wasm_smart(
            marketplace,
            &MarketplaceQueryMsg::ReverseBidsSortedByPrice {
                collection: collection_address.to_string(),
                start_before: None,
                limit: Some(PAGINATION_DEFAULT_LIMIT), // max amount per single query
            },
        )?;

        let CollectionBidsResponse {
            bids: collection_bids,
        } = deps.querier.query_wasm_smart(
            marketplace,
            &MarketplaceQueryMsg::ReverseCollectionBidsSortedByPrice {
                collection: collection_address.to_string(),
                start_before: None,
                limit: Some(PAGINATION_DEFAULT_LIMIT),
            },
        )?;

        let mut offers: Vec<LiquidationBid> = vec![];

        for Bid {
            price,
            bidder,
            token_id,
            ..
        } in bids
        {
            offers.push(LiquidationBid {
                creation_date: block_time,
                liquidator: bidder,
                token_id_list: vec![token_id.to_string()],
                amount: price,
                discount: Decimal::zero(), // TODO: order_info.finders_fee_bps?
                bid_type: BidType::TokenOffer,
            })
        }

        for CollectionBid { price, bidder, .. } in collection_bids {
            let collateral: Vec<Collateral> = deps.querier.query_wasm_smart(
                lending_platform,
                &LendingPlatformQueryMsg::QueryCollateral {
                    collection_address: collection_address.to_string(),
                },
            )?;

            // TODO: check if it's fine to pass all tokens including tokens owned by liquidators
            let token_id_list = collateral
                .into_iter()
                .map(|x| x.token_id_list)
                .flatten()
                .collect();

            offers.push(LiquidationBid {
                creation_date: block_time,
                liquidator: bidder,
                token_id_list,
                amount: price,
                discount: Decimal::zero(), // TODO: order_info.finders_fee_bps?
                bid_type: BidType::CollectionOffer,
            })
        }

        total_offers.push(QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address,
            liquidation_bids: offers,
        });
    }

    Ok(total_offers)
}
