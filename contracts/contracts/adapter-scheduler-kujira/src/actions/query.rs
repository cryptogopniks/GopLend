use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::adapters::scheduler::kujira::{
    state::{CONFIG, LOG},
    types::Config,
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_log(deps: Deps, _env: Env, amount: u32) -> StdResult<Vec<(u64, u32)>> {
    Ok(LOG
        .iter(deps.storage)?
        .rev()
        .take(amount as usize)
        .flatten()
        .collect())
}
