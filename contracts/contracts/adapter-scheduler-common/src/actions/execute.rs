use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        scheduler::common::{
            state::{CONFIG, LOG, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
            types::{ClockSource, Config, Log, TransferAdminState},
        },
    },
    error::ContractError,
    lending_platform::types::LiquidationItem,
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

// offchain handler
pub fn try_push(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    targets: Vec<LiquidationItem>,
    route: Route,
) -> Result<Response, ContractError> {
    let block_time = env.block.time.seconds();
    let sender = info.sender;
    let Config {
        lending_platform_main,
        lending_platform_stable,
        offchain_clock,
        clock_source,
        execution_cooldown,
        ..
    } = CONFIG.load(deps.storage)?;

    if let ClockSource::Onchain = clock_source {
        Err(ContractError::Unauthorized)?;
    }

    if let ClockSource::Offchain = clock_source {
        if !unwrap_field(offchain_clock, "offchain_clock")?.contains(&sender) {
            Err(ContractError::Unauthorized)?;
        }
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
            sender,
            counter,
        },
    )?;

    // select lending platform depending on route
    let lending_platform = match route {
        Route::Empty => unwrap_field(lending_platform_main, "lending_platform_main")?,
        _ => unwrap_field(lending_platform_stable, "lending_platform_stable")?,
    };

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lending_platform.to_string(),
        msg: to_json_binary(
            &goplend_base::lending_platform::msg::ExecuteMsg::Liquidate { targets, route },
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
    lending_platform_main: Option<String>,
    lending_platform_stable: Option<String>,
    offchain_clock: Option<Vec<String>>,
    clock_source: Option<ClockSource>,
    execution_cooldown: Option<u64>,
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

    if let Some(x) = lending_platform_main {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform_main = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = lending_platform_stable {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform_stable = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = offchain_clock {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.offchain_clock = Some(
            x.into_iter()
                .map(|y| deps.api.addr_validate(&y))
                .collect::<StdResult<Vec<Addr>>>()?,
        );
        is_config_updated = true;
    }

    if let Some(x) = clock_source {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.clock_source = x;
        is_config_updated = true;
    }

    if let Some(x) = execution_cooldown {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
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
