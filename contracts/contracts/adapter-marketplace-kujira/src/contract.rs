#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};

use goplend_base::{
    adapters::marketplace::kujira::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    error::ContractError,
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
        ExecuteMsg::AcceptAdminRole {} => e::try_accept_admin_role(deps, env, info),

        ExecuteMsg::UpdateConfig {
            admin,
            worker,
            lending_platform,
        } => e::try_update_config(deps, env, info, admin, worker, lending_platform),

        ExecuteMsg::AcceptBids {
            bidded_collateral_item_list,
        } => e::try_accept_bids(deps, env, info, bidded_collateral_item_list),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::QueryLiquidationBidsByCollectionAddressList { start_after, limit } => {
            to_json_binary(&q::query_liquidation_bids_by_collection_address_list(
                deps,
                env,
                start_after,
                limit,
            )?)
        }

        QueryMsg::QueryLiquidationBidsByCollectionAddress { address } => to_json_binary(
            &q::query_liquidation_bids_by_collection_address(deps, env, address)?,
        ),

        QueryMsg::QueryLiquidationBidsByLiquidatorAddressList { start_after, limit } => {
            to_json_binary(&q::query_liquidation_bids_by_liquidator_address_list(
                deps,
                env,
                start_after,
                limit,
            )?)
        }

        QueryMsg::QueryLiquidationBidsByLiquidatorAddress { address } => to_json_binary(
            &q::query_liquidation_bids_by_liquidator_address(deps, env, address)?,
        ),
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
