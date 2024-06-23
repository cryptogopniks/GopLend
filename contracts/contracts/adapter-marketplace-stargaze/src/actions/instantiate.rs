use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    adapters::marketplace::stargaze::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME, PRICE_UPDATE_PERIOD},
        types::Config,
    },
    assets::Currency,
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
    let main_currency = Currency::new(
        &msg.main_currency.token.verify(deps.api)?,
        msg.main_currency.decimals,
    );

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
                .transpose()?,
            oracle: msg.oracle.map(|x| deps.api.addr_validate(&x)).transpose()?,
            marketplace: msg
                .marketplace
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            dex: msg.dex.map(|x| deps.api.addr_validate(&x)).transpose()?,
            main_currency,
            price_update_period: msg.price_update_period.unwrap_or(PRICE_UPDATE_PERIOD),
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
