use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use goplend_base::{
    error::ContractError,
    lending_platform::{state::TRANSFER_ADMIN_TIMEOUT, types::TransferAdminState},
    oracle::{
        state::{CONFIG, PRICES, TRANSFER_ADMIN_STATE, UPDATE_DATE},
        types::Config,
    },
    utils::{check_authorization, check_funds, AuthType, FundsType},
};

// update prices for specified collections
pub fn try_update_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: Vec<(String, u32)>,
) -> Result<Response, ContractError> {
    let response = Response::new().add_attribute("action", "try_update_prices");
    let block_time = env.block.time.seconds();
    let Config {
        controller,
        execution_cooldown,
        ..
    } = CONFIG.load(deps.storage)?;

    if !controller.contains(&info.sender) {
        Err(ContractError::Unauthorized)?;
    }

    if block_time < UPDATE_DATE.load(deps.storage)? + execution_cooldown {
        Err(ContractError::EarlyExecution)?;
    }

    UPDATE_DATE.save(deps.storage, &block_time)?;

    if data.is_empty() {
        return Ok(response);
    }

    for (postfix, price) in &data {
        PRICES.save(deps.storage, postfix, price)?;
    }

    Ok(response)
}

// clear prices for specified collections
pub fn try_remove_prices(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collections: Vec<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let Config { controller, .. } = CONFIG.load(deps.storage)?;

    if !controller.contains(&sender_address) {
        Err(ContractError::Unauthorized)?;
    }

    for postfix in collections {
        PRICES.remove(deps.storage, &postfix);
    }

    Ok(Response::new().add_attribute("action", "try_remove_prices"))
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
        &Some(worker),
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
    controller: Option<Vec<String>>,
    execution_cooldown: Option<u64>,
    max_price_update_period: Option<u64>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;
    let current_worker = &Some(config.worker.clone());

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
        config.worker = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    if let Some(x) = controller {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::AdminOrWorker,
        )?;
        config.controller = x
            .into_iter()
            .map(|y| deps.api.addr_validate(&y))
            .collect::<StdResult<_>>()?;
        is_config_updated = true;
    }

    if let Some(x) = execution_cooldown {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::AdminOrWorker,
        )?;
        config.execution_cooldown = x;
        is_config_updated = true;
    }

    if let Some(x) = max_price_update_period {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::AdminOrWorker,
        )?;
        config.max_price_update_period = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}
