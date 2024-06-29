use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use goplend_base::{
    adapters::scheduler::common::types::Log,
    assets::{Currency, Funds},
    error::ContractError,
    oracle::{
        state::{
            CONFIG, LOG, PRICE_AND_DATE_LIST, PRICE_RATIO, TRANSFER_ADMIN_STATE,
            TRANSFER_ADMIN_TIMEOUT,
        },
        types::{Config, PriceItem, PriceRatioItem, RawPriceItem, TransferAdminState},
    },
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

// clear storage and set prices for specified collections
pub fn try_set_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    price_ratio: Decimal,
    data: Vec<RawPriceItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let Config {
        controller,
        execution_cooldown,
        ..
    } = CONFIG.load(deps.storage)?;

    if !unwrap_field(controller, "controller")?.contains(&sender_address) {
        Err(ContractError::Unauthorized)?;
    }

    if data.is_empty() {
        Err(ContractError::EmptyRawPriceItemVector)?;
    }

    let Log {
        block_time: block_time_prev,
        counter,
        ..
    } = unwrap_field(LOG.back(deps.storage)?, "log")?;
    let counter = if counter == u8::MAX { 0 } else { counter + 1 };

    if block_time < block_time_prev + execution_cooldown {
        Err(ContractError::EarlyExecution)?;
    }

    if LOG.len(deps.storage)? == (u8::MAX - 1) as u32 {
        LOG.pop_front(deps.storage)?;
    }
    LOG.push_back(
        deps.storage,
        &Log {
            block_time,
            sender: sender_address,
            counter,
        },
    )?;

    PRICE_AND_DATE_LIST.clear(deps.storage);

    for RawPriceItem {
        collection_address,
        price,
        usd_price,
    } in data
    {
        if price.amount.is_zero() {
            Err(ContractError::ZerosInPrices)?;
        }

        let collection_address = &deps.api.addr_validate(&collection_address)?;

        let price_token = price.currency.token.verify(deps.api)?;
        let currency = Currency::new(&price_token, price.currency.decimals);
        let price = Funds::new(price.amount, &currency);

        let usd_price_token = usd_price.currency.token.verify(deps.api)?;
        let usd_currency = Currency::new(&usd_price_token, usd_price.currency.decimals);
        let usd_price = Funds::new(usd_price.amount, &usd_currency);

        if let Ok(PriceItem { update_date, .. }) =
            PRICE_AND_DATE_LIST.load(deps.storage, collection_address)
        {
            if update_date == block_time {
                Err(ContractError::CollectionDuplication)?;
            }
        }

        PRICE_AND_DATE_LIST.save(
            deps.storage,
            collection_address,
            &PriceItem {
                address: collection_address.to_owned(),
                price,
                usd_price,
                update_date: block_time,
                is_outdated: false,
            },
        )?;
    }

    PRICE_RATIO.save(
        deps.storage,
        &PriceRatioItem {
            value: price_ratio,
            update_date: block_time,
            is_outdated: false,
        },
    )?;

    Ok(Response::new().add_attributes(vec![("action", "try_set_prices")]))
}

// update prices for specified collections
pub fn try_update_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    price_ratio: Decimal,
    data: Vec<RawPriceItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let Config {
        controller,
        execution_cooldown,
        ..
    } = CONFIG.load(deps.storage)?;

    if !unwrap_field(controller, "controller")?.contains(&sender_address) {
        Err(ContractError::Unauthorized)?;
    }

    if data.is_empty() {
        Err(ContractError::EmptyRawPriceItemVector)?;
    }

    let Log {
        block_time: block_time_prev,
        counter,
        ..
    } = unwrap_field(LOG.back(deps.storage)?, "log")?;
    let counter = if counter == u8::MAX { 0 } else { counter + 1 };

    if block_time < block_time_prev + execution_cooldown {
        Err(ContractError::EarlyExecution)?;
    }

    if LOG.len(deps.storage)? == (u8::MAX - 1) as u32 {
        LOG.pop_front(deps.storage)?;
    }
    LOG.push_back(
        deps.storage,
        &Log {
            block_time,
            sender: sender_address,
            counter,
        },
    )?;

    for RawPriceItem {
        collection_address,
        price,
        usd_price,
    } in data
    {
        if price.amount.is_zero() {
            Err(ContractError::ZerosInPrices)?;
        }

        let collection_address = &deps.api.addr_validate(&collection_address)?;

        let price_token = price.currency.token.verify(deps.api)?;
        let currency = Currency::new(&price_token, price.currency.decimals);
        let price = Funds::new(price.amount, &currency);

        let usd_price_token = usd_price.currency.token.verify(deps.api)?;
        let usd_currency = Currency::new(&usd_price_token, usd_price.currency.decimals);
        let usd_price = Funds::new(usd_price.amount, &usd_currency);

        if let Ok(PriceItem { update_date, .. }) =
            PRICE_AND_DATE_LIST.load(deps.storage, collection_address)
        {
            if update_date == block_time {
                Err(ContractError::CollectionDuplication)?;
            }
        }

        PRICE_AND_DATE_LIST.save(
            deps.storage,
            collection_address,
            &PriceItem {
                address: collection_address.to_owned(),
                price,
                usd_price,
                update_date: block_time,
                is_outdated: false,
            },
        )?;
    }

    PRICE_RATIO.save(
        deps.storage,
        &PriceRatioItem {
            value: price_ratio,
            update_date: block_time,
            is_outdated: false,
        },
    )?;

    Ok(Response::new().add_attributes(vec![("action", "try_update_prices")]))
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

    if !unwrap_field(controller, "controller")?.contains(&sender_address) {
        Err(ContractError::Unauthorized)?;
    }

    for collection_address in collections {
        let collection_address = deps.api.addr_validate(&collection_address)?;
        PRICE_AND_DATE_LIST.remove(deps.storage, &collection_address);
    }

    Ok(Response::new().add_attributes(vec![("action", "try_remove_prices")]))
}

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let TransferAdminState {
        new_admin,
        deadline,
    } = TRANSFER_ADMIN_STATE.load(deps.storage)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
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

    if let Some(x) = admin {
        check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;
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
        check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = controller {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.controller = Some(
            x.into_iter()
                .map(|y| deps.api.addr_validate(&y))
                .collect::<StdResult<Vec<Addr>>>()?,
        );
        is_config_updated = true;
    }

    if let Some(x) = execution_cooldown {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.execution_cooldown = x;
        is_config_updated = true;
    }

    if let Some(x) = max_price_update_period {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
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

fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    let worker = unwrap_field(worker, "worker");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorker => {
            if !((sender == admin) || (worker.is_ok() && sender == worker?)) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::Specified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !is_included {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorkerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (worker.is_ok() && sender == worker?) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
    };

    Ok(())
}
