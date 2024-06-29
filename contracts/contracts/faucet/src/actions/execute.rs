use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

use osmosis_std::types::osmosis::tokenfactory::v1beta1 as OsmosisFactory;

use goplend_base::{
    error::ContractError,
    faucet::state::{CLAIMABLE_AMOUNT, CLAIM_COOLDOWN, CONFIG, LAST_CLAIM_DATE, TOKEN},
    utils::{check_funds, FundsType},
};

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    claimable_amount: Option<Uint128>,
    claim_cooldown: Option<u64>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;

    if sender_address != config.admin {
        Err(ContractError::Unauthorized)?;
    }

    if let Some(x) = claimable_amount {
        config.claimable_amount = x;
        is_config_updated = true;
    }

    if let Some(x) = claim_cooldown {
        config.claim_cooldown = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let now_in_seconds = env.block.time.seconds();
    let creator = &env.contract.address;
    let denom = TOKEN.load(deps.storage)?;
    let amount = coin(CLAIMABLE_AMOUNT, denom);

    let last_claimed = LAST_CLAIM_DATE
        .load(deps.storage, &sender_address)
        .unwrap_or_default();

    if now_in_seconds < last_claimed + CLAIM_COOLDOWN {
        Err(ContractError::ClaimCooldown)?;
    }

    LAST_CLAIM_DATE.save(deps.storage, &sender_address, &now_in_seconds)?;

    let msg_list = vec![
        OsmosisFactory::MsgMint {
            sender: creator.to_string(),
            amount: Some(amount.clone().into()),
            mint_to_address: creator.to_string(),
        }
        .into(),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: sender_address.to_string(),
            amount: vec![amount],
        }),
    ];

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_claim"))
}

pub fn try_claim_for(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address_and_amount_list: Vec<(String, Uint128)>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let creator = &env.contract.address;
    let denom = TOKEN.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if sender_address != config.admin {
        Err(ContractError::Unauthorized)?;
    }

    let total_amount = address_and_amount_list
        .iter()
        .fold(0, |acc, (_address, amount)| acc + amount.u128());

    let mut msg_list: Vec<CosmosMsg> = vec![OsmosisFactory::MsgMint {
        sender: creator.to_string(),
        amount: Some(coin(total_amount, denom.clone()).into()),
        mint_to_address: creator.to_string(),
    }
    .into()];

    for (address, amount) in address_and_amount_list {
        msg_list.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: deps.api.addr_validate(&address)?.to_string(),
            amount: coins(amount.u128(), denom.clone()),
        }));
    }

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_claim_for"))
}
