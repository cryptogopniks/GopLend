use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use goplend_base::{
    assets::{Currency, Funds},
    error::ContractError,
    oracle::{
        state::{CONFIG, PRICE_AND_DATE_LIST, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, RawPriceItem, TransferAdminState},
    },
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

pub fn try_update_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: Vec<RawPriceItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let scheduler = unwrap_field(CONFIG.load(deps.storage)?.scheduler, "scheduler")?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(scheduler)],
        },
    )?;

    if data.is_empty() {
        Err(ContractError::EmptyRawPriceItemVector)?;
    }

    for RawPriceItem {
        collection_address,
        price,
    } in data
    {
        if price.amount.is_zero() {
            Err(ContractError::ZerosInPrices)?;
        }

        let collection_address = deps.api.addr_validate(&collection_address)?;
        let price_token = price.currency.token.verify(deps.api)?;
        let currency = Currency::new(&price_token, price.currency.decimals);
        let price = Funds::new(price.amount, &currency);

        if PRICE_AND_DATE_LIST.has(deps.storage, &collection_address)
            && PRICE_AND_DATE_LIST
                .load(deps.storage, &collection_address)?
                .1
                == block_time
        {
            Err(ContractError::CollectionDuplication)?;
        }

        PRICE_AND_DATE_LIST.save(deps.storage, &collection_address, &(price, block_time))?;
    }

    Ok(Response::new().add_attributes(vec![("action", "try_update_prices")]))
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

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    scheduler: Option<String>,
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

    if let Some(x) = scheduler {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.scheduler = Some(deps.api.addr_validate(&x)?);
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
