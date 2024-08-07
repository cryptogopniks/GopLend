use cosmwasm_std::{Addr, Deps, Env, Order, StdResult};

use cw_storage_plus::Bound;
use goplend_base::minter::{
    state::{CONFIG, CURRENCIES, TOKEN_COUNT},
    types::{Config, CurrencyInfo},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_currency_info(
    deps: Deps,
    _env: Env,
    denom_or_address: String,
) -> StdResult<CurrencyInfo> {
    CURRENCIES.load(deps.storage, &denom_or_address)
}

pub fn query_currency_info_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_from: Option<String>,
) -> StdResult<Vec<CurrencyInfo>> {
    let start_bound = start_from.as_ref().map(|x| Bound::exclusive(x.as_str()));

    Ok(CURRENCIES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .map(|x| {
            let (_, currency_info) = x.unwrap();
            currency_info
        })
        .collect())
}

pub fn query_currency_info_list_by_owner(
    deps: Deps,
    _env: Env,
    owner: String,
    amount: u32,
    start_from: Option<String>,
) -> StdResult<Vec<CurrencyInfo>> {
    let start_bound = start_from.as_ref().map(|x| Bound::exclusive(x.as_str()));

    Ok(CURRENCIES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .flatten()
        .filter(|(_, currency_info)| currency_info.owner == owner)
        .map(|(_, currency_info)| currency_info)
        .collect())
}

pub fn query_token_count_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_from: Option<String>,
) -> StdResult<Vec<(Addr, u16)>> {
    let address;
    let start_bound = match start_from {
        None => None,
        Some(x) => {
            address = deps.api.addr_validate(&x)?;
            Some(Bound::exclusive(&address))
        }
    };

    TOKEN_COUNT
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .collect()
}
