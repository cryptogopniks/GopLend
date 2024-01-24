use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    lending_platform::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME, IS_LOCKED},
        types::Config,
    },
    utils::{validate_attr, Attrs},
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut attrs = Attrs::init("try_instantiate");
    let api = deps.api;

    CONFIG.save(
        deps.storage,
        &Config::new(
            &info.sender,
            &validate_attr(&mut attrs, api, "owner", &msg.owner)?,
            &validate_attr(&mut attrs, api, "oracle", &msg.oracle)?,
        ),
    )?;

    IS_LOCKED.save(deps.storage, &false)?;

    Ok(Response::new().add_attributes(attrs))
}
