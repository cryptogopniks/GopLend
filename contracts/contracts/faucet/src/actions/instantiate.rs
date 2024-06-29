use cosmwasm_std::{Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};
use cw2::set_contract_version;

use osmosis_std::types::osmosis::tokenfactory::v1beta1 as OsmosisFactory;

use goplend_base::{
    error::ContractError,
    faucet::{
        msg::InstantiateMsg,
        state::{CLAIMABLE_AMOUNT, CLAIM_COOLDOWN, CONFIG, CONTRACT_NAME, TOKEN},
        types::Config,
    },
    utils::{check_funds, FundsType},
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let creator = env.contract.address;
    let full_denom = &get_full_denom(&creator, &msg.subdenom);

    CONFIG.save(
        deps.storage,
        &Config {
            admin: sender_address,
            claimable_amount: msg
                .claimable_amount
                .unwrap_or(Uint128::new(CLAIMABLE_AMOUNT)),
            claim_cooldown: msg.claim_cooldown.unwrap_or(CLAIM_COOLDOWN),
        },
    )?;

    TOKEN.save(deps.storage, full_denom)?;

    let msg: CosmosMsg = OsmosisFactory::MsgCreateDenom {
        sender: creator.to_string(),
        subdenom: msg.subdenom.to_string(),
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_instantiate"))
}

fn get_full_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{creator}/{subdenom}")
}
