use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    scheduler::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME, EXECUTION_COOLDOWN, LOG, LOG_COUNTER},
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

    let offchain_clock = match msg.offchain_clock {
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
            lending_platform: msg
                .lending_platform
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?
                .unwrap_or(sender.to_owned()),
            offchain_clock: offchain_clock.unwrap_or_default(),
            execution_cooldown: msg.execution_cooldown.unwrap_or(EXECUTION_COOLDOWN),
        },
    )?;

    LOG.push_back(
        deps.storage,
        &Log {
            block_time,
            sender: sender.to_owned(),
            counter: LOG_COUNTER,
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
