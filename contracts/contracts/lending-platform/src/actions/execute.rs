use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, Uint128, WasmMsg,
};

use goplend_base::{
    assets::{Currency, TokenUnverified},
    error::ContractError,
    lending_platform::{
        state::{
            BGL_CURRENCY_DECIMALS, CONFIG, CW20_NAME, CW20_SYMBOL, IS_LOCKED,
            SAVE_CW20_ADDRESS_REPLY,
        },
        types::Config,
    },
    utils::{check_funds, unwrap_field, Attrs, AuthType, FundsType},
};

pub fn try_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;

    Ok(Response::new().add_attributes(vec![("action", "try_deposit")]))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    oracle: Option<String>,
    scheduler: Option<String>,
    main_currency: Option<Currency<TokenUnverified>>,
    bgl_currency: Option<Currency<TokenUnverified>>,
    cw20_base_code_id: Option<u64>,
    unbonding_period: Option<u64>,
    borrow_apr: Option<Decimal>,
    borrow_fee_rate: Option<Decimal>,
    liquidation_fee_rate: Option<Decimal>,
    discount_min_rate: Option<Decimal>,
    discount_max_rate: Option<Decimal>,
    bid_min_rate: Option<Decimal>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    let mut attrs = Attrs::init("try_update_config");

    CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
        if let Some(x) = owner {
            config.owner = Some(deps.api.addr_validate(&x)?);
            attrs.push(("owner".to_string(), x));
        }

        if let Some(x) = oracle {
            config.oracle = Some(deps.api.addr_validate(&x)?);
            attrs.push(("oracle".to_string(), x));
        }

        if let Some(x) = scheduler {
            config.scheduler = Some(deps.api.addr_validate(&x)?);
            attrs.push(("scheduler".to_string(), x));
        }

        if let Some(x) = main_currency {
            config.main_currency = Currency::new(&x.token.verify(deps.api)?, x.decimals);
            attrs.push(("main_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = bgl_currency {
            config.bgl_currency = Some(Currency::new(&x.token.verify(deps.api)?, x.decimals));
            attrs.push(("bgl_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = cw20_base_code_id {
            config.cw20_base_code_id = x;
            attrs.push(("cw20_base_code_id".to_string(), x.to_string()));
        }

        if let Some(x) = unbonding_period {
            config.unbonding_period = x;
            attrs.push(("unbonding_period".to_string(), x.to_string()));
        }

        if let Some(x) = borrow_apr {
            config.borrow_apr = x;
            attrs.push(("borrow_apr".to_string(), x.to_string()));
        }

        if let Some(x) = borrow_fee_rate {
            config.borrow_fee_rate = x;
            attrs.push(("borrow_fee_rate".to_string(), x.to_string()));
        }

        if let Some(x) = liquidation_fee_rate {
            config.liquidation_fee_rate = x;
            attrs.push(("liquidation_fee_rate".to_string(), x.to_string()));
        }

        if let Some(x) = discount_min_rate {
            config.discount_min_rate = x;
            attrs.push(("discount_min_rate".to_string(), x.to_string()));
        }

        if let Some(x) = discount_max_rate {
            config.discount_max_rate = x;
            attrs.push(("discount_max_rate".to_string(), x.to_string()));
        }

        if let Some(x) = bid_min_rate {
            config.bid_min_rate = x;
            attrs.push(("bid_min_rate".to_string(), x.to_string()));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
}

pub fn try_create_bgl_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    let creator = env.contract.address;
    let config = CONFIG.load(deps.storage)?;

    if config.bgl_currency.is_some() {
        Err(ContractError::AddressExists)?;
    }

    let cw20_msg = cw20_base::msg::InstantiateMsg {
        name: format!("{}", CW20_NAME),
        symbol: CW20_SYMBOL.to_string(),
        decimals: BGL_CURRENCY_DECIMALS,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: creator.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(creator.to_string()),
        code_id: config.cw20_base_code_id,
        label: format!("CryptoGopniks {}", CW20_SYMBOL),
        msg: to_json_binary(&cw20_msg)?,
        funds: vec![],
    });

    let submsg = SubMsg::reply_on_success(msg, SAVE_CW20_ADDRESS_REPLY);

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attributes([("action", "try_create_cw20")]))
}

fn check_lockout(deps: Deps) -> StdResult<()> {
    if IS_LOCKED.load(deps.storage)? {
        Err(ContractError::ContractIsLocked)?;
    }

    Ok(())
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
