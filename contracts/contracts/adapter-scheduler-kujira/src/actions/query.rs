use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::adapters::scheduler::kujira::{
    state::{CONFIG, LOG},
    types::{Config, Log},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_log(deps: Deps, _env: Env) -> StdResult<Vec<Log>> {
    Ok(LOG.iter(deps.storage)?.rev().flatten().collect())
}
