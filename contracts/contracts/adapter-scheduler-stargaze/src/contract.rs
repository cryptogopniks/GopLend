#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};

use goplend_base::{
    adapters::scheduler::stargaze::msg::{
        ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg,
    },
    error::ContractError,
};

use crate::actions::{execute as e, instantiate::try_instantiate, other as o, query as q};

/// Creates a new contract with the specified parameters packed in the "msg" variable
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    try_instantiate(deps, env, info, msg)
}

/// Exposes all the execute functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Push {} => e::try_push(deps, env, info),

        ExecuteMsg::UpdateConfig {
            lending_platform,
            offchain_clock,
            clock_source,
            min_clock_period,
        } => e::try_update_config(
            deps,
            env,
            info,
            lending_platform,
            offchain_clock,
            clock_source,
            min_clock_period,
        ),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::QueryLog { amount } => to_json_binary(&q::query_log(deps, env, amount)?),
    }
}

/// Exposes all the replies available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    let Reply { id: _, result: _ } = reply;

    Err(ContractError::UndefinedReplyId)
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    o::migrate_contract(deps, env, msg)
}

/// Exposes all functions that can be called only by Cosmos SDK modules
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::BeginBlock {} => o::try_push(deps, env, msg),
        SudoMsg::EndBlock {} => o::try_push(deps, env, msg),
    }
}
