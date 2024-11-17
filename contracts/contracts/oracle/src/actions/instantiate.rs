use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    oracle::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME, EXECUTION_COOLDOWN, MAX_PRICE_UPDATE_PERIOD, UPDATE_DATE},
        types::Config,
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

    CONFIG.save(
        deps.storage,
        &Config {
            admin: sender.to_owned(),
            worker: msg
                .worker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?
                .unwrap_or(sender.to_owned()),
            controller: msg
                .controller
                .map(|x| {
                    x.iter()
                        .map(|y| deps.api.addr_validate(y))
                        .collect::<StdResult<_>>()
                })
                .transpose()?
                .unwrap_or(vec![sender.to_owned()]),
            execution_cooldown: msg.execution_cooldown.unwrap_or(EXECUTION_COOLDOWN),
            max_price_update_period: msg
                .max_price_update_period
                .unwrap_or(MAX_PRICE_UPDATE_PERIOD),
        },
    )?;

    UPDATE_DATE.save(deps.storage, &block_time)?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
