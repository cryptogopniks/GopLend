use cosmwasm_std::{Deps, Env, StdResult};

use goplend_base::faucet::{
    state::{CONFIG, LAST_CLAIM_DATE, TOKEN},
    types::Config,
};

// QueryMsg::QueryLastClaimDate { address } => {
//     to_json_binary(&q::last_claimed(deps, env, address)?)
// }

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_denom(deps: Deps, _env: Env) -> StdResult<String> {
    TOKEN.load(deps.storage)
}

pub fn query_last_claim_date(deps: Deps, _env: Env, address: String) -> StdResult<u64> {
    LAST_CLAIM_DATE.load(deps.storage, &deps.api.addr_validate(&address)?)
}
