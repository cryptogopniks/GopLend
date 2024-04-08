#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use goplend_base::{
    error::ContractError,
    minter_kujira::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use kujira::KujiraMsg;

use crate::actions::{
    execute as e, instantiate::try_instantiate, other::migrate_contract, query as q,
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
) -> Result<Response<KujiraMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateNative {
            token_owner,
            subdenom,
            decimals,
        } => e::try_create_native(deps, env, info, token_owner, subdenom, decimals),

        ExecuteMsg::Mint {
            token,
            amount,
            recipient,
        } => e::try_mint(deps, env, info, token, amount, recipient),

        ExecuteMsg::Burn {} => e::try_burn(deps, env, info, None, None),

        ExecuteMsg::RegisterCurrency { currency, creator } => {
            e::try_register_currency(deps, env, info, currency, creator)
        }
        ExecuteMsg::UpdateConfig { owner } => e::try_update_config(deps, env, info, owner),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryCurrenciesByCreator { creator } => {
            to_json_binary(&q::query_currencies_by_creator(deps, env, creator)?)
        }

        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
