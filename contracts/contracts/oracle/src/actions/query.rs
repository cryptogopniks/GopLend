use cosmwasm_std::{Deps, Env, Order, StdResult, Uint128};

use cw_storage_plus::Bound;
use goplend_base::oracle::{
    state::{CONFIG, DECIMALS_MULTIPLIER, PRICES, UPDATE_DATE},
    types::{get_collection, split_address, Config, PriceItem, QueryPricesResponse},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_prices(
    deps: Deps,
    env: Env,
    amount: u32,
    start_after: Option<String>,
    collections: Option<Vec<String>>,
) -> StdResult<QueryPricesResponse> {
    let block_time = env.block.time.seconds();
    let decimals_multiplier = Uint128::new(DECIMALS_MULTIPLIER);
    let Config {
        max_price_update_period,
        ..
    } = CONFIG.load(deps.storage)?;

    let is_outdated = block_time > UPDATE_DATE.load(deps.storage)? + max_price_update_period;

    if let Some(x) = collections {
        let data = x
            .iter()
            .map(|collection| {
                let (prefix, postfix) = split_address(collection);
                let price = PRICES.load(deps.storage, &postfix)?;

                Ok(PriceItem {
                    collection: get_collection(&prefix, &postfix),
                    price: Uint128::from(price) * decimals_multiplier,
                })
            })
            .collect::<StdResult<_>>()?;

        return Ok(QueryPricesResponse { is_outdated, data });
    }

    let (prefix, _) = split_address(env.contract.address);
    let start_bound = start_after.as_ref().map(|x| Bound::exclusive(x.as_str()));

    let data = PRICES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .map(|x| {
            let (postfix, price) = x.unwrap();

            PriceItem {
                collection: get_collection(&prefix, &postfix),
                price: Uint128::from(price) * decimals_multiplier,
            }
        })
        .collect();

    Ok(QueryPricesResponse { is_outdated, data })
}

pub fn query_block_time(_deps: Deps, env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}
