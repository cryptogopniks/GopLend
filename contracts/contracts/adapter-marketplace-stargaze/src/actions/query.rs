use cosmwasm_std::{Decimal, Deps, Env, StdResult, Uint128};

use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};

use sg_marketplace::{
    msg::{BidsResponse, CollectionBidsResponse, ParamsResponse, QueryMsg as MarketplaceQueryMsg},
    state::{Bid, CollectionBid, Order},
};

use goplend_base::{
    adapters::marketplace::stargaze::{
        state::{CONFIG, PAGINATION_DEFAULT_LIMIT},
        types::{Config, PlatformType},
    },
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::{
        msg::{
            QueryCollectionsResponseItem, QueryLiquidationBidsByCollectionAddressListResponseItem,
            QueryMsg as LendingPlatformQueryMsg,
        },
        types::{BidType, Collateral, LiquidationBid},
    },
    oracle::types::PriceRatioItem,
    utils::unwrap_field,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_offers_by_collection_address_list(
    deps: Deps,
    env: Env,
    platform_type: PlatformType,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
    let block_time = env.block.time.seconds();
    let Config {
        lending_platform_main,
        lending_platform_stable,
        oracle,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(oracle, "oracle")?;
    let marketplace = &unwrap_field(marketplace, "marketplace")?;

    let lending_platform = match platform_type {
        PlatformType::Main => unwrap_field(lending_platform_main, "lending_platform_main")?,
        PlatformType::Stable => unwrap_field(lending_platform_stable, "lending_platform_stable")?,
    };

    let price_ratio = match platform_type {
        PlatformType::Main => Decimal::one(),
        PlatformType::Stable => {
            let PriceRatioItem {
                value, is_outdated, ..
            } = deps.querier.query_wasm_smart(
                oracle,
                &goplend_base::oracle::msg::QueryMsg::QueryPriceRatio {},
            )?;

            if is_outdated {
                Err(ContractError::OutdatedPrice {
                    val: value.to_string(),
                })?;
            }

            value
        }
    };

    // get network fee rate
    let params_response: ParamsResponse = deps
        .querier
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})?;
    let network_fee_rate = params_response.params.trading_fee_percent / u128_to_dec(100u128);

    let collection_list: Vec<QueryCollectionsResponseItem> = deps.querier.query_wasm_smart(
        &lending_platform,
        &LendingPlatformQueryMsg::QueryCollectionList { start_after, limit },
    )?;

    let mut total_offers: Vec<QueryLiquidationBidsByCollectionAddressListResponseItem> = vec![];

    for QueryCollectionsResponseItem {
        address: collection_address,
        ..
    } in collection_list
    {
        // TODO: check if it's sorted in proper order

        // get collection royalty
        let collection_info = deps.querier.query_wasm_smart::<CollectionInfoResponse>(
            collection_address.clone(),
            &Sg721QueryMsg::CollectionInfo {},
        );
        let royalty_fee_rate = match collection_info {
            // cw721
            Err(_) => Decimal::zero(),
            // sg721
            Ok(x) => x.royalty_info.map(|x| x.share).unwrap_or_default(),
        };
        let discount_base = royalty_fee_rate + network_fee_rate;

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

        for bid in bids {
            // TODO: do we need to query more offers if we got too much expired offers?
            // skip expired bids
            if bid.is_expired(&env.block) {
                continue;
            }

            let Bid {
                price,
                bidder,
                token_id,
                finders_fee_bps,
                ..
            } = bid;

            // discount = royalty_fee_rate + network_fee_rate + finders_fee_rate
            let finders_fee_rate = u128_to_dec(finders_fee_bps.unwrap_or_default() / 100);

            offers.push(LiquidationBid {
                creation_date: block_time,
                liquidator: bidder,
                token_id_list: vec![token_id.to_string()],
                amount: calc_price(price, price_ratio),
                discount: discount_base + finders_fee_rate,
                bid_type: BidType::TokenOffer,
            })
        }

        for collection_bid in collection_bids {
            // skip expired bids
            if collection_bid.is_expired(&env.block) {
                continue;
            }

            let CollectionBid {
                price,
                bidder,
                finders_fee_bps,
                ..
            } = collection_bid;

            let collateral: Vec<Collateral> = deps.querier.query_wasm_smart(
                &lending_platform,
                &LendingPlatformQueryMsg::QueryCollateral {
                    collection_address: collection_address.to_string(),
                },
            )?;

            // it's fine to pass all tokens including tokens owned by liquidators
            // as we need offers capacity as much as possible
            let token_id_list = collateral
                .into_iter()
                .flat_map(|x| {
                    x.token_item_list
                        .into_iter()
                        .map(|x| x.id)
                        .collect::<Vec<String>>()
                })
                .collect();

            // discount = royalty_fee_rate + network_fee_rate + finders_fee_rate
            let finders_fee_rate = u128_to_dec(finders_fee_bps.unwrap_or_default() / 100);

            offers.push(LiquidationBid {
                creation_date: block_time,
                liquidator: bidder,
                token_id_list,
                amount: calc_price(price, price_ratio),
                discount: discount_base + finders_fee_rate,
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

// imagine we have 500 STARS offer, 1 STARS is 0.01 USDC
// if we will assume the offer is 5 USDC we will lose some funds due to
// swap fee, price impact, price ratio fluctuation
// expecting 2 % losses we can accept 4.9 USDC offer
//
// price = base_price * price_ratio * (1 - price_impact)
// as price_impact comes with price_ratio
// price = base_price * price_ratio
fn calc_price(base_price: Uint128, price_ratio: Decimal) -> Uint128 {
    (u128_to_dec(base_price) * price_ratio).to_uint_floor()
}
