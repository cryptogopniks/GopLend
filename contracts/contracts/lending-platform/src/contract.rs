#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult,
};

use cw20::Cw20ReceiveMsg;

use goplend_base::{
    error::ContractError,
    lending_platform::{
        msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg},
        state::SAVE_CW20_ADDRESS_REPLY,
    },
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

/// Exposes all execute functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender,
            amount,
            msg,
        }) => match from_json(msg)? {
            ExecuteMsg::Deposit {} => e::try_deposit(deps, env, info, Some(sender), Some(amount)),

            ExecuteMsg::Unbond {} => e::try_unbond(deps, env, info, Some(sender), Some(amount)),

            ExecuteMsg::Repay {} => unimplemented!(),

            ExecuteMsg::PlaceBid { collections } => unimplemented!(),

            ExecuteMsg::UpdateBid {
                collections,
                creation_date,
                amount,
            } => unimplemented!(),

            ExecuteMsg::DepositReserveLiquidity {} => {
                e::try_deposit_reserve_liquidity(deps, env, info, Some(sender), Some(amount))
            }

            ExecuteMsg::AcceptProposal { id } => {
                e::try_accept_proposal(deps, env, info, id, Some(sender), Some(amount))
            }

            _ => Err(ContractError::WrongMessageType)?,
        },

        ExecuteMsg::Deposit {} => e::try_deposit(deps, env, info, None, None),

        ExecuteMsg::Unbond {} => e::try_unbond(deps, env, info, None, None),

        ExecuteMsg::Withdraw {} => e::try_withdraw(deps, env, info),

        ExecuteMsg::DepositCollateral { collections } => {
            e::try_deposit_collateral(deps, env, info, collections)
        }

        ExecuteMsg::WithdrawCollateral { collections } => unimplemented!(),

        ExecuteMsg::Borrow { amount } => unimplemented!(),

        ExecuteMsg::Repay {} => unimplemented!(),

        ExecuteMsg::PlaceBid { collections } => unimplemented!(),

        ExecuteMsg::RemoveBid {
            collections,
            creation_date,
        } => unimplemented!(),

        ExecuteMsg::UpdateBid {
            collections,
            creation_date,
            amount,
        } => unimplemented!(),

        ExecuteMsg::ClaimCollateral {} => unimplemented!(),

        ExecuteMsg::UpdateConfig {
            owner,
            oracle,

            main_currency,
            bgl_currency,
            cw20_base_code_id,
            unbonding_period,
            borrow_apr,
            borrow_fee_rate,
            liquidation_fee_rate,
            discount_min_rate,
            discount_max_rate,
            bid_min_rate,
        } => e::try_update_config(
            deps,
            env,
            info,
            owner,
            oracle,
            main_currency,
            bgl_currency,
            cw20_base_code_id,
            unbonding_period,
            borrow_apr,
            borrow_fee_rate,
            liquidation_fee_rate,
            discount_min_rate,
            discount_max_rate,
            bid_min_rate,
        ),

        ExecuteMsg::CreateBglToken {} => e::try_create_bgl_token(deps, env, info),

        ExecuteMsg::DepositReserveLiquidity {} => {
            e::try_deposit_reserve_liquidity(deps, env, info, None, None)
        }

        ExecuteMsg::Lock {} => e::try_lock(deps, env, info),

        ExecuteMsg::Unlock {} => e::try_unlock(deps, env, info),

        ExecuteMsg::DistributeFunds {
            address_and_weight_list,
        } => e::try_distribute_funds(deps, env, info, address_and_weight_list),

        ExecuteMsg::RemoveCollection { address } => {
            e::try_remove_collection(deps, env, info, address)
        }

        ExecuteMsg::CreateProposal { proposal } => {
            e::try_create_proposal(deps, env, info, proposal)
        }

        ExecuteMsg::RejectProposal { id } => e::try_reject_proposal(deps, env, info, id),

        ExecuteMsg::AcceptProposal { id } => {
            e::try_accept_proposal(deps, env, info, id, None, None)
        }
    }
}

/// Exposes all query functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::QueryPlatformRevenue {} => to_json_binary(&q::query_platform_revenue(deps, env)?),

        QueryMsg::QueryBalances {} => to_json_binary(&q::query_balances(deps, env)?),

        QueryMsg::QueryDepositors { addresses } => {
            to_json_binary(&q::query_depositors(deps, env, addresses)?)
        }

        QueryMsg::QueryBorrowers { addresses } => {
            to_json_binary(&q::query_borrowers(deps, env, addresses)?)
        }

        QueryMsg::QueryLiquidators { addresses } => {
            to_json_binary(&q::query_liquidators(deps, env, addresses)?)
        }

        QueryMsg::QueryCollaterals {
            collection_addresses,
        } => to_json_binary(&q::query_collaterals(deps, env, collection_addresses)?),

        QueryMsg::QueryLiquidationBidsByCollectionAddresses { addresses } => to_json_binary(
            &q::query_liquidation_bids_by_collection_addresses(deps, env, addresses)?,
        ),

        QueryMsg::QueryLiquidationBidsByLiquidatorAddresses { addresses } => to_json_binary(
            &q::query_liquidation_bids_by_liquidator_addresses(deps, env, addresses)?,
        ),

        QueryMsg::QueryProposals { last_amount } => {
            to_json_binary(&q::query_proposals(deps, env, last_amount)?)
        }

        QueryMsg::QueryCollections { addresses } => {
            to_json_binary(&q::query_collections(deps, env, addresses)?)
        }

        QueryMsg::QueryBglCurrencyToMainCurrencyPrice {} => {
            to_json_binary(&q::query_bgl_currency_to_main_currency_price(deps, env)?)
        }

        QueryMsg::QueryDepositApr {} => to_json_binary(&q::query_deposit_apr(deps, env)?),

        QueryMsg::QueryBlockCounter {} => to_json_binary(&q::query_block_counter(deps, env)?),
    }
}

/// Exposes all reply functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let Reply { id, result } = reply;

    match id {
        SAVE_CW20_ADDRESS_REPLY => o::save_cw20_address(deps, env, &result),
        _ => Err(ContractError::UndefinedReplyId),
    }
}

/// Exposes all migration functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    o::migrate_contract(deps, env, msg)
}

/// Exposes all functions that can be called only by Cosmos SDK modules
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::BeginBlock {} => o::try_process(deps, env, msg),
        SudoMsg::EndBlock {} => o::try_process(deps, env, msg),
    }
}
