use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    adapters::scheduler::kujira::{
        msg::InstantiateMsg,
        state::{CLOCK_SOURCE, CONFIG, CONTRACT_NAME, LOG, LOG_COUNTER, MIN_CLOCK_PERIOD},
        types::Config,
    },
    error::ContractError,
    utils::{validate_attr, Attrs},
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut attrs = Attrs::init("try_instantiate");
    let api = deps.api;
    let block_time = env.block.time.seconds();

    CONFIG.save(
        deps.storage,
        &Config {
            admin: info.sender,
            lending_platform: validate_attr(
                &mut attrs,
                api,
                "lending_platform",
                &msg.lending_platform,
            )?,
            offchain_clock: validate_attr(&mut attrs, api, "offchain_clock", &msg.offchain_clock)?,
            clock_source: msg.clock_source.unwrap_or(CLOCK_SOURCE),
            min_clock_period: msg.min_clock_period.unwrap_or(MIN_CLOCK_PERIOD),
        },
    )?;

    LOG.push_back(deps.storage, &(block_time, LOG_COUNTER))?;

    Ok(Response::new().add_attributes(attrs))
}
