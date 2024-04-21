use cosmwasm_std::{Addr, Deps, Env, Order, StdResult};

use goplend_base::minter_kujira::{
    state::{CONFIG, OWNERS},
    types::{Config, QueryCurrenciesFromCreatorResponse},
};

pub fn query_currencies_by_creator(
    deps: Deps,
    _env: Env,
    creator: String,
) -> StdResult<QueryCurrenciesFromCreatorResponse> {
    let creator = deps.api.addr_validate(&creator)?;
    let currencies = OWNERS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .filter(|(_denom, (_currency, owner))| owner == creator)
        .map(|(_denom, (currency, _owner))| currency)
        .collect();

    Ok(QueryCurrenciesFromCreatorResponse { currencies })
}

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_token_owner(deps: Deps, _env: Env, denom: String) -> StdResult<Addr> {
    let (_, owner) = OWNERS.load(deps.storage, &denom)?;
    Ok(owner)
}
