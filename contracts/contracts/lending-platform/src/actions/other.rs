use cosmwasm_std::{DepsMut, Env, Response, StdResult, Storage, SubMsgResult};
use cw2::{get_contract_version, set_contract_version};

use semver::Version;

use goplend_base::{
    assets::{Currency, Token},
    error::ContractError,
    lending_platform::{
        msg::MigrateMsg,
        state::{BGL_CURRENCY_DECIMALS, CONFIG, CONTRACT_NAME},
        types::Config,
    },
    utils::unwrap_field,
};

pub fn save_cw20_address(
    deps: DepsMut,
    _env: Env,
    result: &SubMsgResult,
) -> Result<Response, ContractError> {
    let res = result
        .to_owned()
        .into_result()
        .map_err(|e| ContractError::CustomError { val: e })?;

    let instantiate_event = unwrap_field(
        res.events.iter().find(|x| x.ty == "instantiate"),
        "instantiate_event",
    )?;

    let cw20_address = &unwrap_field(
        instantiate_event
            .attributes
            .iter()
            .find(|x| x.key == "_contract_address"),
        "cw20_address",
    )?
    .value;

    let bgl_currency = Currency::new(
        &Token::new_cw20(&deps.api.addr_validate(cw20_address)?),
        BGL_CURRENCY_DECIMALS,
    );

    CONFIG.update(deps.storage, |mut x| -> StdResult<Config> {
        x.bgl_currency = Some(bgl_currency);
        Ok(x)
    })?;

    Ok(Response::new().add_attributes([("cw20_address", cw20_address)]))
}

pub fn migrate_contract(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    let (version_previous, version_new) = get_versions(deps.storage, msg)?;

    if version_new >= version_previous {
        set_contract_version(deps.storage, CONTRACT_NAME, version_new.to_string())?;
    }

    Ok(Response::new())
}

fn get_versions(
    storage: &dyn Storage,
    msg: MigrateMsg,
) -> Result<(Version, Version), ContractError> {
    let version_previous: Version = get_contract_version(storage)?
        .version
        .parse()
        .map_err(|_| ContractError::ParsingPrevVersion)?;

    let version_new: Version = env!("CARGO_PKG_VERSION")
        .parse()
        .map_err(|_| ContractError::ParsingNewVersion)?;

    if version_new.to_string() != msg.version {
        Err(ContractError::ImproperMsgVersion)?;
    }

    Ok((version_previous, version_new))
}
