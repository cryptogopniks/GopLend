#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult,
};

use goplend_base::{
    error::ContractError,
    minter::{
        msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
        state::SAVE_CW20_ADDRESS_REPLY,
    },
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
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender,
            amount,
            msg,
        }) => match from_json(msg)? {
            ExecuteMsg::Burn {} => e::try_burn(deps, env, info, Some(sender), Some(amount)),

            _ => Err(ContractError::WrongMessageType)?,
        },

        ExecuteMsg::AcceptAdminRole {} => e::try_accept_admin_role(deps, env, info),

        ExecuteMsg::AcceptTokenOwnerRole {} => e::try_accept_token_owner_role(deps, env, info),

        ExecuteMsg::Pause {} => e::try_pause(deps, env, info),

        ExecuteMsg::Unpause {} => e::try_unpause(deps, env, info),

        ExecuteMsg::UpdateConfig {
            admin,
            whitelist,
            cw20_code_id,
            permissionless_token_creation,
            permissionless_token_registration,
            max_tokens_per_owner,
        } => e::try_update_config(
            deps,
            env,
            info,
            admin,
            whitelist,
            cw20_code_id,
            permissionless_token_creation,
            permissionless_token_registration,
            max_tokens_per_owner,
        ),

        ExecuteMsg::CreateNative {
            owner,
            whitelist,
            permissionless_burning,
            subdenom,
            decimals,
        } => e::try_create_native(
            deps,
            env,
            info,
            owner,
            whitelist,
            permissionless_burning,
            subdenom,
            decimals,
        ),

        ExecuteMsg::CreateCw20 {
            owner,
            whitelist,
            permissionless_burning,
            cw20_code_id,
            name,
            symbol,
            decimals,
            marketing,
        } => e::try_create_cw20(
            deps,
            env,
            info,
            owner,
            whitelist,
            permissionless_burning,
            cw20_code_id,
            name,
            symbol,
            decimals,
            marketing,
        ),

        ExecuteMsg::RegisterNative {
            denom,
            owner,
            whitelist,
            permissionless_burning,
            decimals,
        } => e::try_register_native(
            deps,
            env,
            info,
            denom,
            owner,
            whitelist,
            permissionless_burning,
            decimals,
        ),

        ExecuteMsg::RegisterCw20 {
            address,
            owner,
            whitelist,
            permissionless_burning,
            cw20_code_id,
            decimals,
        } => e::try_register_cw20(
            deps,
            env,
            info,
            address,
            owner,
            whitelist,
            permissionless_burning,
            cw20_code_id,
            decimals,
        ),

        ExecuteMsg::UpdateCurrencyInfo {
            denom_or_address,
            owner,
            whitelist,
            permissionless_burning,
        } => e::try_update_currency_info(
            deps,
            env,
            info,
            denom_or_address,
            owner,
            whitelist,
            permissionless_burning,
        ),

        ExecuteMsg::UpdateMetadataNative { denom, metadata } => {
            e::try_update_metadata_native(deps, env, info, denom, metadata)
        }

        ExecuteMsg::UpdateMetadataCw20 {
            address,
            project,
            description,
            logo,
        } => e::try_update_metadata_cw20(deps, env, info, address, project, description, logo),

        ExecuteMsg::ExcludeNative { denom } => e::try_exclude_native(deps, env, info, denom),

        ExecuteMsg::ExcludeCw20 { address } => e::try_exclude_cw20(deps, env, info, address),

        ExecuteMsg::UpdateFaucetConfig {
            denom_or_address,
            claimable_amount,
            claim_cooldown,
        } => e::try_update_faucet_config(
            deps,
            env,
            info,
            denom_or_address,
            claimable_amount,
            claim_cooldown,
        ),

        ExecuteMsg::Mint {
            denom_or_address,
            amount,
            recipient,
        } => e::try_mint(deps, env, info, denom_or_address, amount, recipient),

        ExecuteMsg::MintMultiple {
            denom_or_address,
            account_and_amount_list,
        } => e::try_mint_multiple(deps, env, info, denom_or_address, account_and_amount_list),

        ExecuteMsg::Burn {} => e::try_burn(deps, env, info, None, None),

        ExecuteMsg::Claim { denom_or_address } => e::try_claim(deps, env, info, denom_or_address),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&q::query_config(deps, env)?),

        QueryMsg::FaucetConfig { denom_or_address } => {
            to_json_binary(&q::query_faucet_config(deps, env, denom_or_address)?)
        }

        QueryMsg::CurrencyInfo { denom_or_address } => {
            to_json_binary(&q::query_currency_info(deps, env, denom_or_address)?)
        }

        QueryMsg::CurrencyInfoList {
            amount,
            start_after,
        } => to_json_binary(&q::query_currency_info_list(
            deps,
            env,
            amount,
            start_after,
        )?),

        QueryMsg::CurrencyInfoListByOwner {
            owner,
            amount,
            start_after,
        } => to_json_binary(&q::query_currency_info_list_by_owner(
            deps,
            env,
            owner,
            amount,
            start_after,
        )?),

        QueryMsg::TokenCountList {
            amount,
            start_after,
        } => to_json_binary(&q::query_token_count_list(deps, env, amount, start_after)?),

        QueryMsg::LastClaimDate {
            user,
            denom_or_address,
        } => to_json_binary(&q::query_last_claim_date(
            deps,
            env,
            user,
            denom_or_address,
        )?),

        QueryMsg::Balances { account } => to_json_binary(&q::query_balances(deps, env, account)?),
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
