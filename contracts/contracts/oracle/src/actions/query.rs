use cosmwasm_std::{Addr, Deps, Env, Order, StdResult, Uint128};

use cw_storage_plus::Bound;
use goplend_base::oracle::{
    state::{CONFIG, DECIMALS_MULTIPLIER, PRICES, UPDATE_DATE},
    types::{Config, PriceItem, QueryPricesResponse},
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
    let prefix = get_prefix(env);
    let decimals_multiplier = Uint128::new(DECIMALS_MULTIPLIER);
    let Config {
        max_price_update_period,
        ..
    } = CONFIG.load(deps.storage)?;

    let is_outdated = block_time > UPDATE_DATE.load(deps.storage)? + max_price_update_period;

    if let Some(x) = collections {
        let data = x
            .iter()
            .map(|postfix| {
                let price = PRICES.load(deps.storage, postfix)?;

                Ok(PriceItem {
                    collection: get_collection(&prefix, &postfix),
                    price: Uint128::from(price) * decimals_multiplier,
                })
            })
            .collect::<StdResult<_>>()?;

        return Ok(QueryPricesResponse { is_outdated, data });
    }

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

fn get_prefix(env: Env) -> String {
    let contract_address = env.contract.address.to_string();
    let (prefix, _) = contract_address.split_once('1').unwrap();
    prefix.to_string()
}

fn get_collection(prefix: &str, postfix: &str) -> Addr {
    Addr::unchecked(format!("{}1{}", prefix, postfix))
}
