use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
    SubMsg, SubMsgResult, Uint128, WasmMsg,
};

use goplend_base::{
    assets::TokenUnverified,
    error::ContractError,
    market_maker::{
        hub::state::{
            ADD_OUTPOST_REPLY, CONFIG, IS_LOCKED, OUTPOSTS, OWNERS, TRANSFER_ADMIN_STATE,
            TRANSFER_ADMIN_TIMEOUT, UNDISTRIBUTED_LIQUIDITY,
        },
        outpost::state::CONTRACT_NAME,
        types::{CollateralResponse, HubConfig, OffersResponse, TransferAdminState},
    },
    utils::{
        check_authorization, check_funds, get_transfer_msg, unwrap_field, AuthType, FundsType,
    },
};

use super::query::{query_collateral, query_offers};

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let HubConfig { admin, worker, .. } = CONFIG.load(deps.storage)?;
    let TransferAdminState {
        new_admin,
        deadline,
    } = TRANSFER_ADMIN_STATE.load(deps.storage)?;

    check_authorization(
        &sender_address,
        &admin,
        &worker,
        AuthType::Specified {
            allowlist: vec![Some(new_admin)],
        },
    )?;

    if block_time >= deadline {
        Err(ContractError::TransferAdminDeadline)?;
    }

    CONFIG.update(deps.storage, |mut x| -> StdResult<HubConfig> {
        x.admin = sender_address;
        Ok(x)
    })?;

    TRANSFER_ADMIN_STATE.update(deps.storage, |mut x| -> StdResult<TransferAdminState> {
        x.deadline = block_time;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_accept_admin_role"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    marketplace: Option<String>,
    controller: Option<Vec<String>>,
    token: Option<TokenUnverified>,
    max_outpost_amount: Option<u8>,
    outpost_code_id: Option<u64>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;
    let current_worker = &config.worker.clone();

    if let Some(x) = admin {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        let block_time = env.block.time.seconds();
        let new_admin = &deps.api.addr_validate(&x)?;

        TRANSFER_ADMIN_STATE.save(
            deps.storage,
            &TransferAdminState {
                new_admin: new_admin.to_owned(),
                deadline: block_time + TRANSFER_ADMIN_TIMEOUT,
            },
        )?;

        is_config_updated = true;
    }

    if let Some(x) = worker {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = marketplace {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.marketplace = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = controller {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::AdminOrWorker,
        )?;
        config.controller = Some(
            x.iter()
                .map(|y| deps.api.addr_validate(&y))
                .collect::<StdResult<Vec<_>>>()?,
        );
        is_config_updated = true;
    }

    if let Some(x) = token {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.token = x.verify(deps.api)?;
        is_config_updated = true;
    }

    if let Some(x) = max_outpost_amount {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.max_outpost_amount = x;
        is_config_updated = true;
    }

    if let Some(x) = outpost_code_id {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.outpost_code_id = Some(x);
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_add_outpost(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    let outpost_amount = OUTPOSTS.load(deps.storage)?.len() as u8;
    if outpost_amount >= config.max_outpost_amount {
        Err(ContractError::ExceededOutpostMaxAmount)?;
    }

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(config.admin.to_string()),
        code_id: unwrap_field(config.outpost_code_id, "outpost_code_id")?,
        label: CONTRACT_NAME.to_string(),
        msg: to_json_binary(&goplend_base::market_maker::outpost::msg::InstantiateMsg {})?,
        funds: vec![],
    });

    let submsg = SubMsg::reply_on_success(msg, ADD_OUTPOST_REPLY);

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("action", "try_add_outpost"))
}

pub fn add_outpost_reply(
    deps: DepsMut,
    _env: Env,
    result: &SubMsgResult,
) -> Result<Response, ContractError> {
    let res = result
        .to_owned()
        .into_result()
        .map_err(|e| ContractError::CustomError { val: e })?;

    let instantiate_event = unwrap_field(
        res.events.iter().find(|x| x.ty == "instantiate"),
        "instantiate_event",
    )?;

    let outpost_address = &unwrap_field(
        instantiate_event
            .attributes
            .iter()
            .find(|x| x.key == "_contract_address"),
        "outpost_address",
    )?
    .value;

    OUTPOSTS.update(deps.storage, |mut x| -> StdResult<Vec<Addr>> {
        x.push(deps.api.addr_validate(outpost_address)?);
        Ok(x)
    })?;

    Ok(Response::new())
}

pub fn try_lock(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    IS_LOCKED.update(deps.storage, |_| -> StdResult<bool> { Ok(true) })?;

    Ok(Response::new().add_attributes([("action", "try_lock")]))
}

pub fn try_unlock(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    IS_LOCKED.update(deps.storage, |_| -> StdResult<bool> { Ok(false) })?;

    Ok(Response::new().add_attributes([("action", "try_unlock")]))
}

pub fn try_set_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
    owner_list: Vec<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    let collection_address = deps.api.addr_validate(&collection_address)?;
    let owner_list = owner_list
        .iter()
        .map(|x| deps.api.addr_validate(x))
        .collect::<StdResult<Vec<_>>>()?;

    OWNERS.save(deps.storage, &collection_address, &owner_list)?;

    Ok(Response::new().add_attribute("action", "try_set_collections"))
}

pub fn try_remove_collections(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address_list: Vec<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    for collection_address in collection_address_list {
        let collection_address = deps.api.addr_validate(&collection_address)?;
        OWNERS.remove(deps.storage, &collection_address);
    }

    Ok(Response::new().add_attribute("action", "try_remove_collections"))
}

pub fn try_deposit_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    check_lockout(deps.storage)?;
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;

    let collection_address = deps.api.addr_validate(&collection_address)?;
    let config = CONFIG.load(deps.storage)?;
    let owner_list = OWNERS.load(deps.storage, &collection_address)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: owner_list.into_iter().map(Some).collect(),
        },
    )?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != config.token {
        Err(ContractError::WrongAssetType)?;
    }

    UNDISTRIBUTED_LIQUIDITY.update(
        deps.storage,
        &collection_address,
        |x| -> StdResult<Uint128> { Ok(x.unwrap_or_default() + asset_amount) },
    )?;

    Ok(Response::new().add_attribute("action", "try_deposit_liquidity"))
}

pub fn try_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    collection_address: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_withdraw_liquidity");
    check_lockout(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let config = CONFIG.load(deps.storage)?;
    let owner_list = OWNERS.load(deps.storage, &collection_address)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: owner_list.into_iter().map(Some).collect(),
        },
    )?;

    let mut amount_undistributed =
        UNDISTRIBUTED_LIQUIDITY.load(deps.storage, &collection_address)?;

    // we have enough undistributed liquidity
    if amount <= amount_undistributed {
        amount_undistributed -= amount;
        UNDISTRIBUTED_LIQUIDITY.save(deps.storage, &collection_address, &amount_undistributed)?;

        return Ok(response.add_message(get_transfer_msg(&sender_address, amount, &config.token)?));
    }

    // we need to extract liquidity from offers starting from higher
    let OffersResponse {
        mut price_and_outpost_list,
        ..
    } = query_offers(deps.as_ref(), env, collection_address.to_string())?;
    // sort descending by price
    price_and_outpost_list.sort_unstable_by_key(|(price, _)| Uint128::MAX - price);

    for (price, outpost) in price_and_outpost_list {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: outpost.to_string(),
            msg: to_json_binary(
                &goplend_base::market_maker::outpost::msg::ExecuteMsg::RemoveOffer {
                    collection_address: collection_address.to_string(),
                    recipient: Some(sender_address.to_string()),
                },
            )?,
            funds: vec![],
        });
        response = response.add_message(msg);

        amount_undistributed += price;
        if amount_undistributed >= amount {
            amount_undistributed -= amount;
            break;
        }
    }

    UNDISTRIBUTED_LIQUIDITY.save(deps.storage, &collection_address, &amount_undistributed)?;

    Ok(response)
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_withdraw_collateral");
    check_lockout(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection_address)?;
    let config = CONFIG.load(deps.storage)?;
    let owner_list = OWNERS.load(deps.storage, &collection_address)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: owner_list.into_iter().map(Some).collect(),
        },
    )?;

    // we need to extract liquidity from offers starting from higher
    let collateral_response_list =
        query_collateral(deps.as_ref(), env, collection_address.to_string())?;

    for CollateralResponse { outpost, .. } in collateral_response_list {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: outpost.to_string(),
            msg: to_json_binary(
                &goplend_base::market_maker::outpost::msg::ExecuteMsg::WithdrawCollateral {
                    collection_address: collection_address.to_string(),
                    recipient: Some(sender_address.to_string()),
                },
            )?,
            funds: vec![],
        });
        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_update_offers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
    from_to_price_list: Vec<(Uint128, Uint128)>,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_update_offers"))
}

fn check_lockout(storage: &dyn Storage) -> StdResult<()> {
    if IS_LOCKED.load(storage)? {
        Err(ContractError::ContractIsLocked)?;
    }

    Ok(())
}
