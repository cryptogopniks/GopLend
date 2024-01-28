#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult,
};

use cw20::Cw20ReceiveMsg;

use goplend_base::{
    error::ContractError,
    lending_platform::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

use crate::actions::{instantiate::try_instantiate, other::migrate_contract, query as q};

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
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: _,
            amount: _,
            msg,
        }) => match from_json(msg)? {
            ExecuteMsg::Deposit {} => unimplemented!(),

            ExecuteMsg::Unbond {} => unimplemented!(),

            ExecuteMsg::Repay {} => unimplemented!(),

            ExecuteMsg::PlaceBid { collections } => unimplemented!(),

            ExecuteMsg::UpdateBid {
                collection,
                creation_date,
                amount,
                collections,
            } => unimplemented!(),

            ExecuteMsg::AcceptProposal { id } => unimplemented!(),

            _ => Err(ContractError::WrongMessageType)?,
        },

        ExecuteMsg::Deposit {} => unimplemented!(),

        ExecuteMsg::Unbond {} => unimplemented!(),

        ExecuteMsg::Withdraw {} => unimplemented!(),

        ExecuteMsg::DepositCollateral { collections } => unimplemented!(),

        ExecuteMsg::WithdrawCollateral { collections } => unimplemented!(),

        ExecuteMsg::Borrow { amount } => unimplemented!(),

        ExecuteMsg::Repay {} => unimplemented!(),

        ExecuteMsg::PlaceBid { collections } => unimplemented!(),

        ExecuteMsg::RemoveBid {
            collection,
            creation_date,
        } => unimplemented!(),

        ExecuteMsg::UpdateBid {
            collection,
            creation_date,
            amount,
            collections,
        } => unimplemented!(),

        ExecuteMsg::ClaimCollateral {} => unimplemented!(),

        ExecuteMsg::Process {} => unimplemented!(),

        ExecuteMsg::UpdateConfig {
            owner,
            oracle,
            main_currency,
            bgl_currency,
            unbonding_period,
            borrow_apr,
            borrow_fee_rate,
            liquidation_fee_rate,
            discount_min_rate,
            discount_max_rate,
            bid_min_rate,
        } => unimplemented!(),

        ExecuteMsg::CreateBglToken {} => unimplemented!(),

        ExecuteMsg::Lock {} => unimplemented!(),

        ExecuteMsg::Unlock {} => unimplemented!(),

        ExecuteMsg::DistributeFunds {
            address_and_weight_list,
        } => unimplemented!(),

        ExecuteMsg::RemoveCollection { address } => unimplemented!(),

        ExecuteMsg::CreateProposal { proposal } => unimplemented!(),

        ExecuteMsg::RejectProposal { id } => unimplemented!(),

        ExecuteMsg::AcceptProposal { id } => unimplemented!(),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),
        _ => unimplemented!(),
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
