use cosmwasm_std::{Addr, Deps, Env, Order, StdResult};

use goplend_base::{
    assets::{Funds, Token},
    oracle::{
        state::{CONFIG, PRICE_AND_DATE_LIST},
        types::{Config, PriceItem},
    },
    utils::filter_by_address_list,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

// TODO: use proper pagination
pub fn query_prices(
    deps: Deps,
    _env: Env,
    collection_addresses: Option<Vec<String>>,
) -> StdResult<Vec<PriceItem>> {
    let price_and_date_list: Vec<(Addr, (Funds<Token>, u64))> = PRICE_AND_DATE_LIST
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(
        filter_by_address_list(deps.api, &collection_addresses, &price_and_date_list)?
            .into_iter()
            .map(|(address, (price, date))| PriceItem {
                address,
                price,
                price_update_date: date,
            })
            .collect(),
    )
}

pub fn query_block_time(_deps: Deps, env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}
