#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};

use goplend_base::{
    error::ContractError,
    market_maker::outpost::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetOffer { collection_address } => {
            e::try_set_offer(deps, env, info, collection_address)
        }

        ExecuteMsg::RemoveOffer {
            collection_address,
            recipient,
        } => e::try_remove_offer(deps, env, info, collection_address, recipient),

        ExecuteMsg::WithdrawCollateral {
            collection_address,
            recipient,
        } => e::try_withdraw_collateral(deps, env, info, collection_address, recipient),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryCollateralList { start_after, limit } => {
            to_json_binary(&q::query_collateral_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryCollateral { collection_address } => {
            to_json_binary(&q::query_collateral(deps, env, collection_address)?)
        }

        QueryMsg::QueryOfferList { start_after, limit } => {
            to_json_binary(&q::query_offer_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryOffer { collection_address } => {
            to_json_binary(&q::query_offer(deps, env, collection_address)?)
        }
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
    migrate_contract(deps, env, msg)
}
