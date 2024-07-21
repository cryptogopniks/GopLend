use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    oracle::{
        msg::InstantiateMsg,
        state::{
            CONFIG, CONTRACT_NAME, EXECUTION_COOLDOWN, LOG, LOG_COUNTER, MAX_PRICE_UPDATE_PERIOD,
        },
        types::{Config, Log},
    },
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let sender = &info.sender;
    let block_time = env.block.time.seconds();

    let controller = match msg.controller {
        None => None,
        Some(x) => Some(
            x.into_iter()
                .map(|y| deps.api.addr_validate(&y))
                .collect::<StdResult<Vec<Addr>>>()?,
        ),
    };

    CONFIG.save(
        deps.storage,
        &Config {
            admin: sender.to_owned(),
            worker: msg
                .worker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            controller,
            execution_cooldown: msg.execution_cooldown.unwrap_or(EXECUTION_COOLDOWN),
            max_price_update_period: msg
                .max_price_update_period
                .unwrap_or(MAX_PRICE_UPDATE_PERIOD),
        },
    )?;

    LOG.push_back(
        deps.storage,
        &Log {
            block_time,
            sender: sender.to_owned(),
            outdated_price_collections: vec![],
            counter: LOG_COUNTER,
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
