use cosmwasm_std::{Addr, QuerierWrapper, StdResult};

use sg_index_query::QueryOptions;
use stargaze_marketplace_v2::{
    msg::PriceOffset,
    orders::{Bid, CollectionBid},
};

use goplend_base::{
    adapters::marketplace::stargaze::types::MarketplaceVersion,
    lending_platform::types::BiddedCollateralItem,
};

const BIDS_LIMIT: u32 = 20;

pub fn get_collection_bids(
    querier: &QuerierWrapper,
    marketplace: &Addr,
    collection: impl ToString,
    denom: &str,
) -> StdResult<Vec<CollectionBid>> {
    let query_options: Option<QueryOptions<PriceOffset>> = Some(QueryOptions {
        limit: Some(BIDS_LIMIT),
        descending: Some(true),
        min: None,
        max: None,
    });

    querier.query_wasm_smart(
        marketplace,
        &stargaze_marketplace_v2::msg::QueryMsg::CollectionBidsByPrice {
            collection: collection.to_string(),
            denom: denom.to_string(),
            query_options,
        },
    )
}

pub fn get_bids(
    querier: &QuerierWrapper,
    marketplace: &Addr,
    collection: impl ToString,
    token_id: &str,
    denom: &str,
) -> StdResult<Vec<Bid>> {
    let query_options: Option<QueryOptions<PriceOffset>> = Some(QueryOptions {
        limit: Some(BIDS_LIMIT),
        descending: Some(true),
        min: None,
        max: None,
    });

    querier.query_wasm_smart(
        marketplace,
        &stargaze_marketplace_v2::msg::QueryMsg::BidsByTokenPrice {
            collection: collection.to_string(),
            token_id: token_id.to_string(),
            denom: denom.to_string(),
            query_options,
        },
    )
}

/// returns list of (optional_offer_id, bidded_collateral_item)
pub fn wrap_collateral_list(
    querier: &QuerierWrapper,
    collection_list: &[String],
    bidded_collateral_item_list: &[BiddedCollateralItem],
    marketplace_version: MarketplaceVersion,
    marketplace: &Addr,
    denom: &str,
) -> Vec<(Option<String>, BiddedCollateralItem)> {
    match marketplace_version {
        MarketplaceVersion::V1 => bidded_collateral_item_list
            .iter()
            .map(|x| (None, x.to_owned()))
            .collect(),
        MarketplaceVersion::V2 => {
            let collection_and_bid_list: Vec<CollectionBid> = collection_list
                .iter()
                .flat_map(|collection| {
                    get_collection_bids(querier, marketplace, collection, denom)
                        .unwrap_or_default()
                        .iter()
                        .map(|x| x.to_owned())
                        .collect::<Vec<_>>()
                })
                .collect();

            let bid_list: Vec<Bid> = collection_list
                .iter()
                .flat_map(|collection| {
                    bidded_collateral_item_list
                        .iter()
                        .filter(|x| x.collection.as_str() == collection)
                        .flat_map(|x| {
                            get_bids(querier, marketplace, collection, &x.token_item.id, denom)
                                .unwrap_or_default()
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            bidded_collateral_item_list
                .iter()
                .map(|x| {
                    let collection_offer_id = collection_and_bid_list
                        .iter()
                        .find(|bid| {
                            bid.collection == x.collection
                                && bid.details.price.amount == x.bid_amount
                                && bid.creator == x.liquidator
                        })
                        .map(|x| x.id.clone());

                    let offer_id = bid_list
                        .iter()
                        .find(|bid| {
                            bid.collection == x.collection
                                && bid.token_id == x.token_item.id
                                && bid.details.price.amount == x.bid_amount
                                && bid.creator == x.liquidator
                        })
                        .map(|x| x.id.clone());

                    let id = collection_offer_id.map_or(offer_id, Some);

                    (id, x.to_owned())
                })
                .collect()
        }
    }
}
