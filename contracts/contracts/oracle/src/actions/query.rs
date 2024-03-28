use cosmwasm_std::{Deps, Env, Order, StdResult};

use cw_storage_plus::Bound;
use goplend_base::oracle::{
    state::{CONFIG, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT, PRICE_AND_DATE_LIST},
    types::{Config, PriceItem},
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
                let (price, date) = PRICE_AND_DATE_LIST.load(deps.storage, &address)?;

                Ok(PriceItem {
                    address,
                    price,
                    price_update_date: date,
                })
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
            let (address, (price, date)) = x.unwrap();
            PriceItem {
                address,
                price,
                price_update_date: date,
            }
        })
        .collect())
}

pub fn query_block_time(_deps: Deps, env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}
