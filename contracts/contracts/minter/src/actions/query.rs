use cosmwasm_std::{Addr, Deps, Env, Order, QuerierWrapper, StdResult, Uint128};

use cw_storage_plus::Bound;
use goplend_base::minter::{
    state::{CONFIG, CURRENCIES, FAUCET_CONFIG, LAST_CLAIM_DATE, TOKEN_COUNT},
    types::{Config, CurrencyInfo, FaucetConfig},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_faucet_config(
    deps: Deps,
    _env: Env,
    denom_or_address: String,
) -> StdResult<FaucetConfig> {
    Ok(FAUCET_CONFIG
        .load(deps.storage, &denom_or_address)
        .unwrap_or_default())
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
    start_after: Option<String>,
) -> StdResult<Vec<CurrencyInfo>> {
    let start_bound = start_after.as_ref().map(|x| Bound::exclusive(x.as_str()));

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
    start_after: Option<String>,
) -> StdResult<Vec<CurrencyInfo>> {
    let start_bound = start_after.as_ref().map(|x| Bound::exclusive(x.as_str()));

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
    start_after: Option<String>,
) -> StdResult<Vec<(Addr, u16)>> {
    let address;
    let start_bound = match start_after {
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

pub fn query_last_claim_date(
    deps: Deps,
    _env: Env,
    user: String,
    denom_or_address: String,
) -> StdResult<u64> {
    let user = deps.api.addr_validate(&user)?;

    Ok(LAST_CLAIM_DATE
        .load(deps.storage, (&user, &denom_or_address))
        .unwrap_or_default())
}

pub fn query_balances(deps: Deps, _env: Env, account: String) -> StdResult<Vec<(Uint128, String)>> {
    let account = deps.api.addr_validate(&account)?;
    let native_balances = deps.querier.query_all_balances(&account)?;
    let currency_list = CURRENCIES
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, CurrencyInfo)>>>()?;

    let balances: Vec<(Uint128, String)> = currency_list
        .iter()
        .map(|(denom_or_address, currency_info)| {
            let amount = if currency_info.currency.token.is_native() {
                native_balances
                    .iter()
                    .find(|x| &x.denom == denom_or_address)
                    .map(|x| x.amount)
                    .unwrap_or_default()
            } else {
                get_cw20_balance(&deps.querier, &account, &Addr::unchecked(denom_or_address))
                    .unwrap_or_default()
            };

            (amount, denom_or_address.to_owned())
        })
        .filter(|(amount, _)| !amount.is_zero())
        .collect();

    Ok(balances)
}

fn get_cw20_balance(
    querier: &QuerierWrapper,
    owner: impl ToString,
    token: &Addr,
) -> StdResult<Uint128> {
    let cw20::BalanceResponse { balance } = querier.query_wasm_smart(
        token,
        &cw20::Cw20QueryMsg::Balance {
            address: owner.to_string(),
        },
    )?;

    Ok(balance)
}
