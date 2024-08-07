use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    minter::{
        msg::InstantiateMsg,
        state::{CONFIG, CONTRACT_NAME, IS_PAUSED, MAX_TOKENS_PER_OWNER},
        types::Config,
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

    IS_PAUSED.save(deps.storage, &false)?;
    CONFIG.save(
        deps.storage,
        &Config {
            admin: sender.to_owned(),
            whitelist: msg
                .whitelist
                .unwrap_or(vec![sender.to_string()])
                .iter()
                .map(|x| deps.api.addr_validate(x))
                .collect::<StdResult<Vec<Addr>>>()?,
            cw20_code_id: msg.cw20_code_id,
            permissionless_token_creation: msg.permissionless_token_creation.unwrap_or_default(),
            permissionless_token_registration: msg
                .permissionless_token_registration
                .unwrap_or_default(),
            max_tokens_per_owner: msg.max_tokens_per_owner.unwrap_or(MAX_TOKENS_PER_OWNER),
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
