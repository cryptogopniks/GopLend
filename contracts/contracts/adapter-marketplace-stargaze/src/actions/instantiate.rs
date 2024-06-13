use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    adapters::marketplace::stargaze::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME},
        types::Config,
    },
    error::ContractError,
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let sender = &info.sender;

    CONFIG.save(
        deps.storage,
        &Config {
            admin: sender.to_owned(),
            worker: msg
                .worker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            lending_platform: msg
                .lending_platform
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            marketplace: msg
                .marketplace
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            infinity_pool: msg
                .infinity_pool
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
