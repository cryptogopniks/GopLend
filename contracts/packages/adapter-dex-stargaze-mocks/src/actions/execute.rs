use cosmwasm_std::{
    coins, Addr, BankMsg, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use goplend_base::{
    adapters::dex::stargaze::{
        state::{CONFIG, MAX_PRICE_IMPACT, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, Route, TransferAdminState},
    },
    converters::{str_to_dec, u128_to_dec},
    error::ContractError,
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

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
    lending_platform_stable: Option<String>,
    adapter_marketplace: Option<String>,
    recover_address: Option<String>,
    max_price_impact: Option<Decimal>,
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

    if let Some(x) = lending_platform_stable {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform_stable = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = adapter_marketplace {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.adapter_marketplace = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = recover_address {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.recover_address = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = max_price_impact {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.max_price_impact = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_swap(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _route: Route,
    price_ratio: Decimal,
) -> Result<Response, ContractError> {
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    let Config {
        lending_platform_stable,
        adapter_marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![adapter_marketplace],
        },
    )?;

    let lending_platform = &unwrap_field(lending_platform_stable, "lending_platform")?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    let max_price_impact = str_to_dec(MAX_PRICE_IMPACT);

    // min_token_out_amount = token_in_amount * price_ratio * (1 - max_price_impact)
    let min_token_out_amount =
        (u128_to_dec(asset_amount) * price_ratio * (Decimal::one() - max_price_impact))
            .to_uint_floor()
            .u128();

    let denom_in = asset_info.try_get_native()?;
    let denom = if denom_in == "ustars" {
        "uusdc"
    } else if denom_in == "ukuji" {
        "uusk"
    } else {
        unimplemented!()
    };

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: lending_platform.to_string(),
        amount: coins(min_token_out_amount, denom),
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_swap"))
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
