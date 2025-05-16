use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw_gopniks::{
    assets::{Funds, InfoResp},
    auth::{Auth, TransferAdminState},
};

use goplend_base::{
    error::ContractError,
    lending_platform::{state::TRANSFER_ADMIN_TIMEOUT, types::LiquidationItem},
    scheduler::{
        state::{CONFIG, LOG},
        types::{Config, Log},
    },
};

// offchain handler
pub fn try_push(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    targets: Vec<LiquidationItem>,
) -> Result<Response, ContractError> {
    let block_time = env.block.time.seconds();
    let sender = info.sender;
    let Config {
        lending_platform,
        offchain_clock,
        ..
    } = CONFIG.load(deps.storage)?;
    Auth::specified(deps.api, &offchain_clock)?.assert(&sender)?;

    let Log { counter, .. } = LOG.back(deps.storage)?.unwrap();
    let counter = if counter == u8::MAX { 0 } else { counter + 1 };

    if LOG.len(deps.storage)? == u8::MAX as u32 + 1 {
        LOG.pop_front(deps.storage)?;
    }
    LOG.push_back(
        deps.storage,
        &Log {
            block_time,
            sender,
            counter,
        },
    )?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lending_platform.to_string(),
        msg: to_json_binary(
            &goplend_base::lending_platform::msg::ExecuteMsg::Liquidate { targets },
        )?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_push"))
}

pub fn try_accept_admin_role(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    TransferAdminState::accept_admin(&mut deps, &env, &sender)?;

    CONFIG.update(deps.storage, |mut x| -> StdResult<_> {
        x.admin = sender;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_accept_admin_role"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    lending_platform: Option<String>,
    offchain_clock: Option<Vec<String>>,
    execution_cooldown: Option<u64>,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;
    Auth::simple(current_admin).assert(&sender)?;

    if let Some(x) = admin {
        TransferAdminState::update_admin(
            &mut deps,
            &env,
            &sender,
            &current_admin,
            &x,
            TRANSFER_ADMIN_TIMEOUT,
        )?;

        is_config_updated = true;
    }

    if let Some(x) = worker {
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = lending_platform {
        config.lending_platform = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    if let Some(x) = offchain_clock {
        config.offchain_clock = x
            .into_iter()
            .map(|y| deps.api.addr_validate(&y))
            .collect::<StdResult<Vec<Addr>>>()?;
        is_config_updated = true;
    }

    if let Some(x) = execution_cooldown {
        config.execution_cooldown = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}
