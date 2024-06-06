use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::{
    adapters::marketplace::kujira::{state::CONFIG, types::Config},
    lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_offers_by_collection_address_list(
    _deps: Deps,
    _env: Env,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
    unimplemented!()
}
