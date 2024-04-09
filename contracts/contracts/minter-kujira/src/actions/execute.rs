use cosmwasm_std::{
    Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};

use kujira::{DenomMsg, KujiraMsg};

use goplend_base::{
    assets::{Currency, Token, TokenUnverified},
    error::ContractError,
    minter_kujira::{
        state::{CONFIG, DEFAULT_DECIMALS, OWNERS},
        types::Config,
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
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

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
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

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

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response<KujiraMsg>, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_update_config");
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
        if let Some(x) = owner {
            config.owner = Some(deps.api.addr_validate(&x)?);
        }

        Ok(config)
    })?;

    // fake msg to get proper response
    if env.block.time.seconds() < 10 {
        let owners = OWNERS
            .range(deps.storage, None, None, Order::Ascending)
            .take(1)
            .flatten()
            .collect::<Vec<(String, (Currency<Token>, Addr))>>();
        let (denom, (_, owner)) = owners.first().ok_or(ContractError::AssetIsNotFound)?;

        let msg: CosmosMsg<KujiraMsg> =
            CosmosMsg::Custom(KujiraMsg::Denom(DenomMsg::ChangeAdmin {
                denom: denom.into(),
                address: owner.to_owned(),
            }));

        response = response.add_message(msg);
    }

    Ok(response)
}

fn get_full_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{creator}/{subdenom}")
}

fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let Config { admin, owner, .. } = CONFIG.load(deps.storage)?;
    let owner = unwrap_field(owner, "owner");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrOwner => {
            if !((sender == admin) || (owner.is_ok() && sender == owner?)) {
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
        AuthType::AdminOrOwnerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (owner.is_ok() && sender == owner?) || is_included) {
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
