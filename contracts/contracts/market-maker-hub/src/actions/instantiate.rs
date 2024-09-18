use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    market_maker::{
        hub::{
            msg::InstantiateMsg,
            state::{CONFIG, CONTRACT_NAME, IS_LOCKED, MAX_OUTPOST_AMOUNT},
        },
        types::HubConfig,
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

    let sender = &info.sender;

    CONFIG.save(
        deps.storage,
        &HubConfig {
            admin: sender.to_owned(),
            worker: msg
                .worker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            marketplace: msg
                .marketplace
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            controller: msg
                .controller
                .map(|x| x.iter().map(|y| deps.api.addr_validate(&y)).collect())
                .transpose()?,
            token: msg.token.verify(deps.api)?,
            max_outpost_amount: msg.max_outpost_amount.unwrap_or(MAX_OUTPOST_AMOUNT),
            outpost_code_id: msg.outpost_code_id,
        },
    )?;

    IS_LOCKED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
