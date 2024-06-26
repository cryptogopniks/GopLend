use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    adapters::marketplace::stargaze::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME},
        types::{Config, MarketplaceVersion},
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
            lending_platform_main: msg
                .lending_platform_main
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            lending_platform_stable: msg
                .lending_platform_stable
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            oracle: msg.oracle.map(|x| deps.api.addr_validate(&x)).transpose()?,
            marketplace: msg
                .marketplace
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            adapter_dex: msg
                .adapter_dex
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            marketplace_version: msg.marketplace_version.unwrap_or(MarketplaceVersion::V1),
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
