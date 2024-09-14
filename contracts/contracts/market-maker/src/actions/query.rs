use cosmwasm_std::{Addr, Decimal, Deps, Env, StdResult, Uint128};

use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};

use sg_marketplace::{
    msg::{
        AsksResponse, BidOffset, BidsResponse, CollectionBidOffset, CollectionBidsResponse,
        ParamsResponse, QueryMsg as MarketplaceQueryMsg,
    },
    state::{Bid, CollectionBid, Order},
};

use goplend_base::{
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::{
        msg::QueryMsg as LendingPlatformQueryMsg,
        types::{BidType, Collateral, CollectionInfo, LiquidationBid},
    },
    market_maker::{
        state::{CONFIG, PAGINATION_DEFAULT_LIMIT},
        types::{CollectionOwners, Config, LiquidityInfo, OffersResponse},
    },
    utils::unwrap_field,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_collection_owners_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollectionOwners<Addr>>> {
    unimplemented!()
}

pub fn query_collection_owners(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<CollectionOwners<Addr>> {
    unimplemented!()
}

pub fn query_liquidity_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<LiquidityInfo>> {
    unimplemented!()
}

pub fn query_liquidity(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<LiquidityInfo> {
    unimplemented!()
}

pub fn query_collateral_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<CollectionInfo<Addr>>> {
    unimplemented!()
}

pub fn query_collateral(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<CollectionInfo<Addr>> {
    unimplemented!()
}

pub fn query_offers_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<OffersResponse>> {
    unimplemented!()
}

pub fn query_offers(
    deps: Deps,
    _env: Env,
    collection_address: String,
) -> StdResult<OffersResponse> {
    unimplemented!()
}
