use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::adapters::scheduler::common::{
    state::{CONFIG, LOG},
    types::{Config, Log},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_log(deps: Deps, _env: Env) -> StdResult<Vec<Log>> {
    LOG.iter(deps.storage)?.rev().collect()
}
