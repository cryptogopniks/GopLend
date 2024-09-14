use cosmwasm_std::{
    coins, to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};

use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;

use goplend_base::{
    assets::TokenUnverified,
    error::ContractError,
    lending_platform::types::CollectionInfo,
    market_maker::{
        state::{CONFIG, IS_LOCKED, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, TransferAdminState},
    },
    utils::{
        check_authorization, check_funds, get_collection_operator_approvals, get_transfer_msg,
        unwrap_field, AuthType, FundsType,
    },
};

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
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
    controller: Option<Vec<String>>,
    token: Option<TokenUnverified>,
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

    if let Some(x) = controller {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::AdminOrWorker,
        )?;
        config.controller = Some(
            x.iter()
                .map(|y| deps.api.addr_validate(&y))
                .collect::<StdResult<Vec<_>>>()?,
        );
        is_config_updated = true;
    }

    if let Some(x) = token {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.token = x.verify(deps.api)?;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_lock(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    IS_LOCKED.update(deps.storage, |_| -> StdResult<bool> { Ok(true) })?;

    Ok(Response::new().add_attributes([("action", "try_lock")]))
}

pub fn try_unlock(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    IS_LOCKED.update(deps.storage, |_| -> StdResult<bool> { Ok(false) })?;

    Ok(Response::new().add_attributes([("action", "try_unlock")]))
}

pub fn try_set_collections(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    list: Vec<CollectionInfo<String>>,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_set_collections"))
}

pub fn try_remove_collections(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    list: Vec<String>,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_remove_collections"))
}

pub fn try_deposit_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_deposit_liquidity"))
}

pub fn try_withdraw_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_withdraw_liquidity"))
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_withdraw_collateral"))
}

pub fn try_update_offers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
    from_to_price_list: Vec<(Uint128, Uint128)>,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_update_offers"))
}
