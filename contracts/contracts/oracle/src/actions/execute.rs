use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use cw_gopniks::{
    assets::{Funds, InfoResp},
    auth::{Auth, TransferAdminState},
};
use goplend_base::{
    error::ContractError,
    lending_platform::state::TRANSFER_ADMIN_TIMEOUT,
    oracle::{
        state::{CONFIG, PRICES, UPDATE_DATE},
        types::Config,
    },
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
    Auth::specified(deps.api, &controller)?.assert(&info.sender)?;

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
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let Config { controller, .. } = CONFIG.load(deps.storage)?;
    Auth::specified(deps.api, &controller)?.assert(&sender)?;

    for postfix in collections {
        PRICES.remove(deps.storage, &postfix);
    }

    Ok(Response::new().add_attribute("action", "try_remove_prices"))
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
    controller: Option<Vec<String>>,
    execution_cooldown: Option<u64>,
    max_price_update_period: Option<u64>,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;
    let current_worker = &Some(config.worker.clone());

    if let Some(x) = admin {
        Auth::simple(current_admin).assert(&sender)?;
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
        Auth::simple(current_admin).assert(&sender)?;
        config.worker = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    if let Some(x) = controller {
        Auth::simple_optional(deps.api, current_admin, current_worker)?.assert(&sender)?;
        config.controller = x
            .into_iter()
            .map(|y| deps.api.addr_validate(&y))
            .collect::<StdResult<_>>()?;
        is_config_updated = true;
    }

    if let Some(x) = execution_cooldown {
        Auth::simple_optional(deps.api, current_admin, current_worker)?.assert(&sender)?;
        config.execution_cooldown = x;
        is_config_updated = true;
    }

    if let Some(x) = max_price_update_period {
        Auth::simple_optional(deps.api, current_admin, current_worker)?.assert(&sender)?;
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
