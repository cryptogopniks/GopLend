use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    QuerierWrapper, Response, StdResult, Uint128, WasmMsg,
};

use sg_index_query::QueryOptions;
use stargaze_marketplace_v2::{
    msg::PriceOffset,
    orders::{CollectionBid, OrderDetails},
};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        marketplace::stargaze::{
            state::{CONFIG, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
            types::{Config, MarketplaceVersion, TransferAdminState},
        },
    },
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::types::{BidType, BiddedCollateralItem, CommonConfig},
    oracle::types::PriceRatioItem,
    utils::{
        check_authorization, check_funds, get_collection_operator_approvals, get_transfer_msg,
        unwrap_field, AuthType, FundsType,
    },
};

pub fn get_collection_bids(
    querier: &QuerierWrapper,
    marketplace: &Addr,
    collection: &Addr,
    denom: &str,
) -> StdResult<Vec<CollectionBid>> {
    const LIMIT: u32 = 20;

    let query_options: Option<QueryOptions<PriceOffset>> = Some(QueryOptions {
        limit: Some(LIMIT),
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
