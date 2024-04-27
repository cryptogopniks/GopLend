use cosmwasm_std::{
    Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use kujira::{DenomMsg, KujiraMsg};

use goplend_base::{
    assets::{Currency, Token, TokenUnverified},
    error::ContractError,
    minter_kujira::{
        state::{CONFIG, DEFAULT_DECIMALS, OWNERS, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, TransferAdminState},
    },
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

pub fn try_create_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_owner: String,
    subdenom: String,
    decimals: Option<u8>,
) -> Result<Response<KujiraMsg>, ContractError> {
    let (sender_address, ..) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let owner = deps.api.addr_validate(&token_owner)?;
    let creator = env.contract.address;

    let full_denom = &get_full_denom(&creator, &subdenom);
    let currency = Currency::new(
        &Token::new_native(full_denom),
        decimals.unwrap_or(DEFAULT_DECIMALS),
    );

    OWNERS.update(
        deps.storage,
        full_denom,
        |x| -> StdResult<(Currency<Token>, Addr)> {
            match x {
                Some(_) => Err(ContractError::DenomExists)?,
                None => Ok((currency, owner)),
            }
        },
    )?;

    let msg: CosmosMsg<KujiraMsg> = CosmosMsg::Custom(KujiraMsg::Denom(DenomMsg::Create {
        subdenom: subdenom.into(),
    }));

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_create_native")]))
}

pub fn try_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: TokenUnverified,
    amount: Uint128,
    recipient: String,
) -> Result<Response<KujiraMsg>, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_mint");
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let token = token.verify(deps.api)?;
    let token_denom = match token.clone() {
        Token::Native { denom } => denom,
        Token::Cw20 { address } => address.to_string(),
    };
    let (_, token_owner) = OWNERS
        .load(deps.storage, &token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    if token.is_native() {
        let msg: CosmosMsg<KujiraMsg> = CosmosMsg::Custom(KujiraMsg::Denom(DenomMsg::Mint {
            denom: token_denom.into(),
            amount,
            recipient: deps.api.addr_validate(&recipient)?,
        }));

        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response<KujiraMsg>, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_burn");
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;

    let token_denom = match asset_info.clone() {
        Token::Native { denom } => denom,
        Token::Cw20 { address } => address.to_string(),
    };
    let (_, token_owner) = OWNERS
        .load(deps.storage, &token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    if asset_info.is_native() {
        let msg: CosmosMsg<KujiraMsg> = CosmosMsg::Custom(KujiraMsg::Denom(DenomMsg::Burn {
            denom: token_denom.into(),
            amount: asset_amount,
        }));

        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_register_currency(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    currency: Currency<TokenUnverified>,
    creator: String,
) -> Result<Response<KujiraMsg>, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let creator = deps.api.addr_validate(&creator)?;
    let token = currency.token.verify(deps.api)?;
    let currency = Currency::new(&token, currency.decimals);
    let token_denom = match token {
        Token::Native { denom } => denom,
        Token::Cw20 { address } => address.to_string(),
    };

    OWNERS.update(
        deps.storage,
        &token_denom,
        |x| -> StdResult<(Currency<Token>, Addr)> {
            match x {
                Some(_) => Err(ContractError::DenomExists)?,
                None => Ok((currency, creator)),
            }
        },
    )?;

    Ok(Response::new().add_attributes([("action", "try_register_currency")]))
}

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response<KujiraMsg>, ContractError> {
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
) -> Result<Response<KujiraMsg>, ContractError> {
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

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_update_token_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
    owner: String,
) -> Result<Response<KujiraMsg>, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let owner = deps.api.addr_validate(&owner)?;
    let (currency, _) = OWNERS.load(deps.storage, &denom)?;
    OWNERS.save(deps.storage, &denom, &(currency, owner))?;

    Ok(Response::new().add_attributes([("action", "try_update_token_owner")]))
}

fn get_full_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{creator}/{subdenom}")
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
