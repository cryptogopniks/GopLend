use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use goplend_base::{
    adapters::scheduler::common::{
        state::{CONFIG, LOG, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, Log, TransferAdminState},
    },
    error::ContractError,
    lending_platform::types::LiquidationItem,
    utils::{check_authorization, check_funds, unwrap_field, AuthType, FundsType},
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

    if !offchain_clock.contains(&sender) {
        Err(ContractError::Unauthorized)?;
    }

    let Log { counter, .. } = unwrap_field(LOG.back(deps.storage)?, "log")?;
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
        .add_attributes(vec![("action", "try_push")]))
}

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    let block_time = env.block.time.seconds();
    let TransferAdminState {
        new_admin,
        deadline,
    } = TRANSFER_ADMIN_STATE.load(deps.storage)?;

    check_authorization(
        &sender_address,
        &admin,
        &worker,
        AuthType::Specified {
            allowlist: vec![Some(new_admin)],
        },
    )?;

    if block_time >= deadline {
        Err(ContractError::TransferAdminDeadline)?;
    }

    CONFIG.update(deps.storage, |mut x| -> StdResult<Config> {
        x.admin = sender_address;
        Ok(x)
    })?;

    TRANSFER_ADMIN_STATE.update(deps.storage, |mut x| -> StdResult<TransferAdminState> {
        x.deadline = block_time;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_accept_admin_role"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    lending_platform: Option<String>,
    offchain_clock: Option<Vec<String>>,
    execution_cooldown: Option<u64>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;
    let current_worker = &config.worker.clone();

    if let Some(x) = admin {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        let block_time = env.block.time.seconds();
        let new_admin = &deps.api.addr_validate(&x)?;

        TRANSFER_ADMIN_STATE.save(
            deps.storage,
            &TransferAdminState {
                new_admin: new_admin.to_owned(),
                deadline: block_time + TRANSFER_ADMIN_TIMEOUT,
            },
        )?;

        is_config_updated = true;
    }

    if let Some(x) = worker {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = lending_platform {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.lending_platform = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    if let Some(x) = offchain_clock {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.offchain_clock = x
            .into_iter()
            .map(|y| deps.api.addr_validate(&y))
            .collect::<StdResult<Vec<Addr>>>()?;
        is_config_updated = true;
    }

    if let Some(x) = execution_cooldown {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
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
