use cosmwasm_std::{Addr, Decimal, QuerierWrapper, StdResult};

use sg_index_query::QueryOptions;
use stargaze_marketplace_v2::{
    msg::PriceOffset,
    orders::{Bid, CollectionBid},
};
use stargaze_royalty_registry::state::RoyaltyProtocol;

use goplend_base::{
    converters::u128_to_dec, lending_platform::types::BiddedCollateralItem, utils::unwrap_field,
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
    marketplace: &Addr,
    denom: &str,
) -> Vec<(String, BiddedCollateralItem)> {
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
            let collection_id = collection_and_bid_list
                .iter()
                .find(|bid| {
                    bid.collection == x.collection
                        && bid.details.price.amount == x.bid_amount
                        && bid.creator == x.liquidator
                })
                .map(|x| x.id.clone());

            let id = bid_list
                .iter()
                .find(|bid| {
                    bid.collection == x.collection
                        && bid.token_id == x.token_item.id
                        && bid.details.price.amount == x.bid_amount
                        && bid.creator == x.liquidator
                })
                .map(|x| x.id.clone());

            let order_id = collection_id.map_or(id, Some).unwrap();

            (order_id, x.to_owned())
        })
        .collect()
}

pub fn get_main_denom(querier: &QuerierWrapper, oracle: &Option<Addr>) -> StdResult<String> {
    let oracle = &unwrap_field(oracle.to_owned(), "oracle")?;

    let goplend_base::oracle::types::Config { main_currency, .. } =
        querier.query_wasm_smart(oracle, &goplend_base::oracle::msg::QueryMsg::QueryConfig {})?;

    main_currency.token.try_get_native()
}

pub fn get_discount_base(
    querier: &QuerierWrapper,
    marketplace: &Addr,
    collection_address: impl ToString,
) -> StdResult<Decimal> {
    // get network fee rate
    let stargaze_marketplace_v2::state::Config::<Addr> {
        protocol_fee_bps,
        royalty_registry,
        ..
    } = querier.query_wasm_smart(
        marketplace,
        &stargaze_marketplace_v2::msg::QueryMsg::Config {},
    )?;
    let network_fee_rate = u128_to_dec(protocol_fee_bps) / u128_to_dec(100_u128);

    // TODO: why we get royalty list?
    // get collection royalty
    let royalty_protocol_list: Vec<RoyaltyProtocol> = querier
        .query_wasm_smart(
            royalty_registry,
            &stargaze_royalty_registry::msg::QueryMsg::RoyaltyProtocolByCollection {
                collection: collection_address.to_string(),
                query_options: Some(QueryOptions {
                    limit: Some(1),
                    descending: Some(true),
                    min: None,
                    max: None,
                }),
            },
        )
        .unwrap_or_default();

    let royalty_fee_rate = royalty_protocol_list
        .first()
        .map(|x| x.royalty_entry.share)
        .unwrap_or_default();
    let discount_base = royalty_fee_rate + network_fee_rate;

    Ok(discount_base)
}
