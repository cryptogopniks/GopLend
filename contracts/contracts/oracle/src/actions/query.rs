use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::oracle::{state::CONFIG, types::Config};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}
