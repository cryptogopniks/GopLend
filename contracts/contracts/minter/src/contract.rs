use cosmwasm_std::from_json;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};

use goplend_base::{
    error::ContractError,
    minter::{
        msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
        state::SAVE_CW20_ADDRESS_REPLY,
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
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender,
            amount,
            msg,
        }) => match from_json(msg)? {
            ExecuteMsg::Burn {} => e::try_burn(deps, env, info, Some(sender), Some(amount)),

            _ => Err(ContractError::WrongMessageType)?,
        },

        ExecuteMsg::CreateNative {
            token_owner,
            subdenom,
            decimals,
        } => e::try_create_native(deps, env, info, token_owner, subdenom, decimals),

        ExecuteMsg::CreateCw20 {
            token_owner,
            name,
            symbol,
            decimals,
            marketing,
        } => e::try_create_cw20(
            deps,
            env,
            info,
            token_owner,
            name,
            symbol,
            decimals,
            marketing,
        ),

        ExecuteMsg::Mint {
            token,
            amount,
            recipient,
        } => e::try_mint(deps, env, info, token, amount, recipient),

        ExecuteMsg::Burn {} => e::try_burn(deps, env, info, None, None),

        ExecuteMsg::SetMetadataNative { token, metadata } => {
            e::try_set_metadata_native(deps, env, info, token, metadata)
        }

        ExecuteMsg::UpdateMarketingCw20 {
            token,
            project,
            description,
            marketing,
        } => e::try_update_marketing_cw20(deps, env, info, token, project, description, marketing),

        ExecuteMsg::UploadLogoCw20 { token, logo } => {
            e::try_upload_logo_cw20(deps, env, info, token, logo)
        }

        ExecuteMsg::ChangeAdminNative { token } => {
            e::try_change_admin_native(deps, env, info, token)
        }

        ExecuteMsg::UpdateMinterCw20 { token, new_minter } => {
            e::try_update_minter_cw20(deps, env, info, token, new_minter)
        }

        ExecuteMsg::RegisterCurrency { currency, creator } => {
            e::try_register_currency(deps, env, info, currency, creator)
        }

        ExecuteMsg::AcceptAdminRole {} => e::try_accept_admin_role(deps, env, info),

        ExecuteMsg::UpdateConfig {
            admin,
            worker,
            cw20_base_code_id,
        } => e::try_update_config(deps, env, info, admin, worker, cw20_base_code_id),

        ExecuteMsg::UpdateTokenOwner { denom, owner } => {
            e::try_update_token_owner(deps, env, info, denom, owner)
        }
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryCurrenciesByCreator { creator } => {
            to_json_binary(&q::query_currencies_by_creator(deps, env, creator)?)
        }

        QueryMsg::QueryConfig {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::QueryTokenOwner { denom } => {
            to_json_binary(&q::query_token_owner(deps, env, denom)?)
        }
    }
}

/// Exposes all reply functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let Reply { id, result } = reply;

    match id {
        SAVE_CW20_ADDRESS_REPLY => e::save_cw20_address(deps, env, &result),
        _ => Err(ContractError::UndefinedReplyId),
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
