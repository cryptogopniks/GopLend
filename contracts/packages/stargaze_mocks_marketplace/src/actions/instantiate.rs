use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};
use cw2::set_contract_version;

use goplend_base::{
    error::ContractError,
    utils::{check_funds, FundsType},
};

use crate::{
    msg::InstantiateMsg,
    state::{COLLECTION_OFFERS, CONTRACT_NAME, TOKEN},
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let (_sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;

    let bids_sum = msg
        .bids
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur.price);

    if asset_amount != bids_sum {
        Err(ContractError::CustomError {
            val: "wrong bids sum".to_string(),
        })?;
    }

    TOKEN.save(deps.storage, &asset_info)?;
    COLLECTION_OFFERS.save(deps.storage, &msg.bids)?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
