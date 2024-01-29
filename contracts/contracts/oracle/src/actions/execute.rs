use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use goplend_base::{
    error::ContractError,
    oracle::{
        state::CONFIG,
        types::{Config, RawPriceItem},
    },
    utils::{check_funds, Attrs, FundsType},
};

pub fn try_update_prices(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: Vec<RawPriceItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    Ok(Response::new().add_attributes(vec![("action", "try_update_prices")]))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    scheduler: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let mut attrs = Attrs::init("try_update_config");

    CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
        if sender_address != config.admin {
            Err(ContractError::Unauthorized)?;
        }

        if let Some(x) = scheduler {
            config.scheduler = Some(deps.api.addr_validate(&x)?);
            attrs.push(("scheduler".to_string(), x));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
}
