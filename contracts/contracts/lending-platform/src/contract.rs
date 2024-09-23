#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};

use cw20::Cw20ReceiveMsg;

use goplend_base::{
    error::ContractError,
    lending_platform::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
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

            ExecuteMsg::Repay {} => e::try_repay(deps, env, info, Some(sender), Some(amount)),

            ExecuteMsg::PlaceBid {
                collections,
                discount,
            } => e::try_place_bid(
                deps,
                env,
                info,
                collections,
                discount,
                Some(sender),
                Some(amount),
            ),

            ExecuteMsg::UpdateBid {
                collections,
                creation_date,
                amount: bid_amount,
                discount,
            } => e::try_update_bid(
                deps,
                env,
                info,
                collections,
                creation_date,
                bid_amount,
                discount,
                Some(sender),
                Some(amount),
            ),

            ExecuteMsg::DepositReserveLiquidity {} => {
                e::try_deposit_reserve_liquidity(deps, env, info, Some(sender), Some(amount))
            }

            ExecuteMsg::ReinforceBglToken {} => {
                e::try_reinforce_bgl_token(deps, env, info, Some(sender), Some(amount))
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

        ExecuteMsg::WithdrawCollateral { collections } => {
            e::try_withdraw_collateral(deps, env, info, collections)
        }

        ExecuteMsg::Borrow { amount } => e::try_borrow(deps, env, info, amount),

        ExecuteMsg::Repay {} => e::try_repay(deps, env, info, None, None),

        ExecuteMsg::PlaceBid {
            collections,
            discount,
        } => e::try_place_bid(deps, env, info, collections, discount, None, None),

        ExecuteMsg::RemoveBid {
            collection_addresses,
            creation_date,
        } => e::try_remove_bid(deps, env, info, collection_addresses, creation_date),

        ExecuteMsg::UpdateBid {
            collections,
            creation_date,
            amount: bid_amount,
            discount,
        } => e::try_update_bid(
            deps,
            env,
            info,
            collections,
            creation_date,
            bid_amount,
            discount,
            None,
            None,
        ),

        ExecuteMsg::Liquidate { targets, route } => {
            e::try_liquidate(deps, env, info, targets, route)
        }

        ExecuteMsg::AcceptAdminRole {} => e::try_accept_admin_role(deps, env, info),

        ExecuteMsg::UpdateAddressConfig {
            admin,
            worker,
            oracle,
            minter,
            adapter_scheduler,
            adapter_marketplace,
        } => e::try_update_address_config(
            deps,
            env,
            info,
            admin,
            worker,
            oracle,
            minter,
            adapter_scheduler,
            adapter_marketplace,
        ),

        ExecuteMsg::UpdateRateConfig {
            borrow_apr,
            borrow_fee_rate,
            liquidation_fee_rate,
            discount_min_rate,
            discount_max_rate,
            bid_min_rate,
        } => e::try_update_rate_config(
            deps,
            env,
            info,
            borrow_apr,
            borrow_fee_rate,
            liquidation_fee_rate,
            discount_min_rate,
            discount_max_rate,
            bid_min_rate,
        ),

        ExecuteMsg::UpdateCommonConfig {
            main_currency,
            bgl_currency,
            unbonding_period,
            collateral_min_value,
            borrowers_reserve_fraction_ratio,
            is_marketplace_interface_enabled,
        } => e::try_update_common_config(
            deps,
            env,
            info,
            main_currency,
            bgl_currency,
            unbonding_period,
            collateral_min_value,
            borrowers_reserve_fraction_ratio,
            is_marketplace_interface_enabled,
        ),

        ExecuteMsg::DepositReserveLiquidity {} => {
            e::try_deposit_reserve_liquidity(deps, env, info, None, None)
        }

        ExecuteMsg::WithdrawReserveLiquidity { amount } => {
            e::try_withdraw_reserve_liquidity(deps, env, info, amount)
        }

        ExecuteMsg::ReinforceBglToken {} => e::try_reinforce_bgl_token(deps, env, info, None, None),

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
        QueryMsg::QueryAddressConfig {} => to_json_binary(&q::query_address_config(deps, env)?),

        QueryMsg::QueryRateConfig {} => to_json_binary(&q::query_rate_config(deps, env)?),

        QueryMsg::QueryCommonConfig {} => to_json_binary(&q::query_common_config(deps, env)?),

        QueryMsg::QueryPlatformRevenue {} => to_json_binary(&q::query_platform_revenue(deps, env)?),

        QueryMsg::QueryBalances {} => to_json_binary(&q::query_balances(deps, env)?),

        QueryMsg::QueryUnbonderList { start_after, limit } => {
            to_json_binary(&q::query_unbonder_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryUnbonder { address } => {
            to_json_binary(&q::query_unbonder(deps, env, address)?)
        }

        QueryMsg::QueryBorrowerList { start_after, limit } => {
            to_json_binary(&q::query_borrower_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryBorrower { address } => {
            to_json_binary(&q::query_borrower(deps, env, address)?)
        }

        QueryMsg::QueryLiquidatorList { start_after, limit } => {
            to_json_binary(&q::query_liquidator_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryLiquidator { address } => {
            to_json_binary(&q::query_liquidator(deps, env, address)?)
        }

        QueryMsg::QueryCollateralList { start_after, limit } => {
            to_json_binary(&q::query_collateral_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryCollateral { collection_address } => {
            to_json_binary(&q::query_collateral(deps, env, collection_address)?)
        }

        QueryMsg::QueryCollateralByOwner { owner } => {
            to_json_binary(&q::query_collateral_by_owner(deps, env, owner)?)
        }

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

        QueryMsg::QueryProposals { start_after, limit } => {
            to_json_binary(&q::query_proposals(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryCollectionList { start_after, limit } => {
            to_json_binary(&q::query_collection_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryCollection { address } => {
            to_json_binary(&q::query_collection(deps, env, address)?)
        }

        QueryMsg::QueryBglCurrencyToMainCurrencyPrice {} => {
            to_json_binary(&q::query_bgl_currency_to_main_currency_price(deps, env)?)
        }

        QueryMsg::QueryConditionalDepositApr {
            amount_to_deposit,
            amount_to_withdraw,
        } => to_json_binary(&q::query_conditional_deposit_apr(
            deps,
            env,
            amount_to_deposit,
            amount_to_withdraw,
        )?),

        QueryMsg::QueryConditionalLtv {
            borrower,
            amount_to_deposit,
            amount_to_withdraw,
            amount_to_borrow,
            amount_to_repay,
        } => to_json_binary(&q::query_conditional_ltv(
            deps,
            env,
            borrower,
            amount_to_deposit,
            amount_to_withdraw,
            amount_to_borrow,
            amount_to_repay,
        )?),

        QueryMsg::QueryLtvList { start_after, limit } => {
            to_json_binary(&q::query_ltv_list(deps, env, start_after, limit)?)
        }

        QueryMsg::QueryTotalAvailableToBorrowLiquidity {} => {
            to_json_binary(&q::query_total_available_to_borrow_liquidity(deps, env)?)
        }

        QueryMsg::QueryAvailableToBorrow {
            borrower,
            target_ltv,
        } => to_json_binary(&q::query_available_to_borrow(
            deps, env, borrower, target_ltv,
        )?),

        QueryMsg::QueryAmounts {} => to_json_binary(&q::query_amounts(deps, env)?),

        QueryMsg::QueryUserInfo { address } => {
            to_json_binary(&q::query_user_info(deps, env, address)?)
        }
    }
}

// /// Exposes all the replies available in the contract
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
//     let Reply { id: _, result: _ } = reply;

//     Err(ContractError::UndefinedReplyId)
// }

/// Exposes all migration functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    o::migrate_contract(deps, env, msg)
}
