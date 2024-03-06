use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    minter::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME},
        types::Config,
    },
    utils::{add_attr, validate_attr, Attrs},
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
            &add_attr(&mut attrs, "cw20_base_code_id", &msg.cw20_base_code_id)?,
        ),
    )?;

    Ok(Response::new().add_attributes(attrs))
}
