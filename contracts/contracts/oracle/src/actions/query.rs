use cosmwasm_std::{Deps, Env, Order, StdResult};

use cw_storage_plus::Bound;
use goplend_base::{
    adapters::scheduler::common::types::Log,
    oracle::{
        state::{CONFIG, LOG, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT, PRICE_AND_DATE_LIST},
        types::{Config, PriceItem},
    },
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_prices(
    deps: Deps,
    _env: Env,
    collection_addresses: Option<Vec<String>>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<PriceItem>> {
    if let Some(x) = collection_addresses {
        return x
            .iter()
            .map(|y| {
                let address = deps.api.addr_validate(y)?;
                PRICE_AND_DATE_LIST.load(deps.storage, &address)
            })
            .collect::<StdResult<Vec<PriceItem>>>();
    }

    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(PRICE_AND_DATE_LIST
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (_address, price_item) = x.unwrap();
            price_item
        })
        .collect())
}

pub fn query_block_time(_deps: Deps, env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}

pub fn query_log(deps: Deps, _env: Env) -> StdResult<Vec<Log>> {
    LOG.iter(deps.storage)?.rev().collect()
}
