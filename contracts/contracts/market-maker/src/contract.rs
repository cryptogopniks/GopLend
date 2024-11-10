#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use goplend_base::{
    error::ContractError,
    market_maker::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
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
        ExecuteMsg::AcceptAdminRole {} => e::try_accept_admin_role(deps, env, info),

        ExecuteMsg::UpdateConfig {
            admin,
            worker,
            controller,
            lending_platform,
            oracle,
            token,
        } => e::try_update_config(
            deps,
            env,
            info,
            admin,
            worker,
            controller,
            lending_platform,
            oracle,
            token,
        ),

        ExecuteMsg::RemoveOffers {
            collection,
            recipient,
        } => e::try_remove_offers(deps, env, info, collection, recipient),

        ExecuteMsg::Pause {} => e::try_pause(deps, env, info),

        ExecuteMsg::Unpause {} => e::try_unpause(deps, env, info),

        ExecuteMsg::SetCollection { collection, owner } => {
            e::try_set_collection(deps, env, info, collection, owner)
        }

        ExecuteMsg::RemoveCollections { collection_list } => {
            e::try_remove_collections(deps, env, info, collection_list)
        }

        ExecuteMsg::DepositLiquidity { collection } => {
            e::try_deposit_liquidity(deps, env, info, collection)
        }

        ExecuteMsg::WithdrawUndistributedLiquidity { collection } => {
            e::try_withdraw_undistributed_liquidity(deps, env, info, collection)
        }

        ExecuteMsg::WithdrawCollateral { collection } => {
            e::try_withdraw_collateral(deps, env, info, collection)
        }

        ExecuteMsg::UpdateOffers {
            collection,
            from_to_price_list,
        } => e::try_update_offers(deps, env, info, collection, from_to_price_list),

        ExecuteMsg::AcceptOffers {
            bidded_collateral_item_list,
        } => e::try_accept_offers(deps, env, info, bidded_collateral_item_list),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        // #[returns(Vec<super::types::OffersResponse>)]
        QueryMsg::QueryOfferPricesList {
            amount,
            start_after,
        } => unimplemented!(),

        // #[returns(super::types::OffersResponse)]
        QueryMsg::QueryOfferPrices { collection: String } => unimplemented!(),

        QueryMsg::QueryCollectionOwnerList {
            amount,
            start_after,
        } => to_json_binary(&q::query_collection_owner_list(
            deps,
            env,
            amount,
            start_after,
        )?),

        QueryMsg::QueryCollectionOwner { collection } => {
            to_json_binary(&q::query_collection_owner(deps, env, collection)?)
        }

        QueryMsg::QueryLiquidityList {
            amount,
            start_after,
        } => to_json_binary(&q::query_liquidity_list(deps, env, amount, start_after)?),

        QueryMsg::QueryLiquidity { collection } => {
            to_json_binary(&q::query_liquidity(deps, env, collection)?)
        }

        QueryMsg::QueryCollateralList {
            amount,
            start_after,
        } => to_json_binary(&q::query_collateral_list(deps, env, amount, start_after)?),

        QueryMsg::QueryCollateral { collection } => {
            to_json_binary(&q::query_collateral(deps, env, collection)?)
        }
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
