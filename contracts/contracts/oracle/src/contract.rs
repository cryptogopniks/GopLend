#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use goplend_base::{
    error::ContractError,
    oracle::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

use crate::actions::{
    execute as e, instantiate::try_instantiate, migrate::migrate_contract, query as q,
};

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
        ExecuteMsg::SetPrices(price_list) => e::try_set_prices(deps, env, info, price_list),

        ExecuteMsg::UpdatePrices(price_list) => e::try_update_prices(deps, env, info, price_list),

        ExecuteMsg::RemovePrices { collections } => {
            e::try_remove_prices(deps, env, info, collections)
        }

        ExecuteMsg::AcceptAdminRole {} => e::try_accept_admin_role(deps, env, info),

        ExecuteMsg::UpdateConfig {
            admin,
            worker,
            controller,
            execution_cooldown,
            max_price_update_period,
            stable_currency,
        } => e::try_update_config(
            deps,
            env,
            info,
            admin,
            worker,
            controller,
            execution_cooldown,
            max_price_update_period,
            stable_currency,
        ),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::QueryPrices {
            amount,
            start_after,
            collections,
        } => to_json_binary(&q::query_prices(
            deps,
            env,
            amount,
            start_after,
            collections,
        )?),

        QueryMsg::QueryBlockTime {} => to_json_binary(&q::query_block_time(deps, env)?),

        QueryMsg::QueryLog {} => to_json_binary(&q::query_log(deps, env)?),
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
