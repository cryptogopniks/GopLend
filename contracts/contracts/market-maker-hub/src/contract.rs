#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};

use goplend_base::{
    error::ContractError,
    market_maker::hub::{
        msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
        state::ADD_OUTPOST_REPLY,
    },
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
            marketplace,
            controller,
            token,
            max_outpost_amount,
            outpost_code_id,
        } => e::try_update_config(
            deps,
            env,
            info,
            admin,
            worker,
            marketplace,
            controller,
            token,
            max_outpost_amount,
            outpost_code_id,
        ),

        ExecuteMsg::AddOutpost {} => e::try_add_outpost(deps, env, info),

        ExecuteMsg::Lock {} => e::try_lock(deps, env, info),

        ExecuteMsg::Unlock {} => e::try_unlock(deps, env, info),

        ExecuteMsg::SetCollection {
            collection_address,
            owner_list,
        } => e::try_set_collection(deps, env, info, collection_address, owner_list),

        ExecuteMsg::RemoveCollections {
            collection_address_list,
        } => e::try_remove_collections(deps, env, info, collection_address_list),

        ExecuteMsg::DepositLiquidity { collection_address } => {
            e::try_deposit_liquidity(deps, env, info, collection_address)
        }

        ExecuteMsg::WithdrawLiquidity {
            amount,
            collection_address,
        } => e::try_withdraw_liquidity(deps, env, info, amount, collection_address),

        ExecuteMsg::WithdrawCollateral { collection_address } => {
            e::try_withdraw_collateral(deps, env, info, collection_address)
        }

        ExecuteMsg::UpdateOffers {
            collection_address,
            from_to_price_list,
        } => e::try_update_offers(deps, env, info, collection_address, from_to_price_list),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::QueryOutposts {} => to_json_binary(&q::query_outposts(deps, env)?),

        QueryMsg::QueryCollectionOwnersList { start_after, limit } => to_json_binary(
            &q::query_collection_owners_list(deps, env, start_after, limit)?,
        ),

        QueryMsg::QueryCollectionOwners { collection_address } => {
            to_json_binary(&q::query_collection_owners(deps, env, collection_address)?)
        }

        QueryMsg::QueryLiquidityList { start_after, limit } => {
            to_json_binary(&q::query_liquidity_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryLiquidity { collection_address } => {
            to_json_binary(&q::query_liquidity(deps, env, collection_address)?)
        }

        QueryMsg::QueryCollateralList { start_after, limit } => {
            to_json_binary(&q::query_collateral_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryCollateral { collection_address } => {
            to_json_binary(&q::query_collateral(deps, env, collection_address)?)
        }
    }
}

/// Exposes all the replies available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let Reply { id, result } = reply;

    match id {
        ADD_OUTPOST_REPLY => e::add_outpost_reply(deps, env, &result),
        _ => Err(ContractError::UndefinedReplyId),
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
