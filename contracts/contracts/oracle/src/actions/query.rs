use cosmwasm_std::{Deps, Env, Order, StdResult, Uint128};

use cw_storage_plus::Bound;
use goplend_base::{
    assets::{Funds, Token},
    oracle::{
        state::{CONFIG, LOG, PRICE_AND_DATE_LIST},
        types::{Config, Log, PriceItem},
    },
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
) -> StdResult<Vec<PriceItem<Funds<Token>>>> {
    let block_time = env.block.time.seconds();
    let Config {
        max_price_update_period,
        stable_currency,
        ..
    } = CONFIG.load(deps.storage)?;
    let decreased_block_time =
        std::cmp::max(block_time, max_price_update_period) - max_price_update_period;

    if let Some(x) = collections {
        return x
            .iter()
            .map(|y| {
                let address = deps.api.addr_validate(y)?;
                let price_item =
                    PRICE_AND_DATE_LIST
                        .load(deps.storage, &address)
                        .unwrap_or(PriceItem {
                            address: address.to_owned(),
                            usd_price: Uint128::zero(),
                            update_date: 0,
                            is_outdated: true,
                        });

                Ok(PriceItem {
                    address,
                    usd_price: Funds::new(price_item.usd_price, &stable_currency),
                    update_date: price_item.update_date,
                    is_outdated: decreased_block_time > price_item.update_date,
                })
            })
            .collect();
    }

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
        .take(amount as usize)
        .map(|x| {
            let (address, price_item) = x.unwrap();

            PriceItem {
                address,
                usd_price: Funds::new(price_item.usd_price, &stable_currency),
                update_date: price_item.update_date,
                is_outdated: decreased_block_time > price_item.update_date,
            }
        })
        .collect())
}

pub fn query_block_time(_deps: Deps, env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}

pub fn query_log(deps: Deps, _env: Env) -> StdResult<Vec<Log>> {
    LOG.iter(deps.storage)?.rev().collect()
}
