#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use stargaze_marketplace_v2::msg::{ExecuteMsg, QueryMsg};

use goplend_base::error::ContractError;

use crate::{
    actions::{execute as e, instantiate::try_instantiate, query as q},
    msg::{InstantiateMsg, SudoMsg},
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
        ExecuteMsg::AcceptCollectionBid {
            id,
            token_id,
            details,
        } => e::accept_collection_bid(deps, env, info, id, token_id, details),

        ExecuteMsg::AcceptBid { id, details } => {
            e::accept_collection_bid(deps, env, info, id, String::default(), details)
        }

        _ => unimplemented!(),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&q::config(deps, env)?),

        _ => unimplemented!(),
    }
}

/// Exposes all functions that can be called only by Cosmos SDK modules
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::SetBids { bids } => e::try_set_bids(deps, env, bids),

        SudoMsg::SetRoyaltyRegistry(royalty_registry) => {
            e::try_set_royalty_registry(deps, env, royalty_registry)
        }
    }
}
