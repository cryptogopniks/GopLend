#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use sg_marketplace::msg::{ExecuteMsg, QueryMsg};

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
            collection,
            token_id,
            bidder,
            finder,
            amount,
        } => e::accept_collection_bid(
            deps, env, info, collection, token_id, bidder, finder, amount,
        ),

        ExecuteMsg::AcceptBid {
            collection,
            token_id,
            bidder,
            finder,
            amount,
        } => e::accept_collection_bid(
            deps, env, info, collection, token_id, bidder, finder, amount,
        ),

        _ => unimplemented!(),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ReverseCollectionBidsSortedByPrice {
            collection,
            start_before,
            limit,
        } => to_json_binary(&q::collection_bids_sorted_by_price(
            deps,
            env,
            collection,
            start_before,
            limit,
        )?),

        QueryMsg::ReverseBidsSortedByPrice {
            collection,
            start_before,
            limit,
        } => to_json_binary(&q::bids_sorted_by_price(
            deps,
            env,
            collection,
            start_before,
            limit,
        )?),

        QueryMsg::Params {} => to_json_binary(&q::params(deps, env)?),

        _ => unimplemented!(),
    }
}

/// Exposes all functions that can be called only by Cosmos SDK modules
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::SetBids { bids } => e::try_set_bids(deps, env, bids),
    }
}
