#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use sg_marketplace::msg::{ExecuteMsg, QueryMsg};

use goplend_base::error::ContractError;

use crate::{
    actions::{execute as e, instantiate::try_instantiate, query as q},
    msg::InstantiateMsg,
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
        } => e::accept_collection_bid(deps, env, info, collection, token_id, bidder, finder),

        _ => unimplemented!(),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CollectionBidsSortedByPrice {
            collection,
            start_after,
            limit,
        } => to_json_binary(&q::collection_bids_sorted_by_price(
            deps,
            env,
            collection,
            start_after,
            limit,
        )?),

        QueryMsg::CollectionBidsByBidder {
            bidder,
            start_after,
            limit,
        } => to_json_binary(&q::collection_bids_by_bidder(
            deps,
            env,
            bidder,
            start_after,
            limit,
        )?),

        _ => unimplemented!(),
    }
}
