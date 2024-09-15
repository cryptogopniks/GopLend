use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    market_maker::{
        outpost::{
            msg::InstantiateMsg,
            state::{CONFIG, CONTRACT_NAME},
        },
        types::OutpostConfig,
    },
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let sender = info.sender;
    CONFIG.save(deps.storage, &OutpostConfig { hub: sender })?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
