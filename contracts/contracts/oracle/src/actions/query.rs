use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::oracle::{msg::QueryPricesResponseItem, state::CONFIG, types::Config};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_prices(
    deps: Deps,
    _env: Env,
    collection_addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryPricesResponseItem>> {
    unimplemented!()
}
