use cosmwasm_std::{
    coins, to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage, SubMsg, SubMsgResult, Uint128, WasmMsg,
};

use goplend_base::{
    assets::TokenUnverified,
    error::ContractError,
    lending_platform::{state::TRANSFER_ADMIN_TIMEOUT, types::TransferAdminState},
    market_maker::{
        state::{CONFIG, IS_PAUSED, OWNERS, TRANSFER_ADMIN_STATE, UNDISTRIBUTED_LIQUIDITY},
        types::Config,
    },
    utils::{
        check_authorization, check_funds, get_transfer_msg, unwrap_field, AuthType, FundsType,
    },
};

use super::query::get_price_and_outpost_list;

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
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

    CONFIG.update(deps.storage, |mut x| -> StdResult<Config> {
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
    controller: Option<Vec<String>>,
    lending_platform: Option<String>,
    oracle: Option<String>,
    token: Option<TokenUnverified>,
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

    if let Some(x) = controller {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.controller = Some(
            x.iter()
                .map(|y| deps.api.addr_validate(y))
                .collect::<StdResult<Vec<_>>>()?,
        );
        is_config_updated = true;
    }

    if let Some(x) = lending_platform {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.lending_platform = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = oracle {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.oracle = Some(deps.api.addr_validate(&x)?);
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

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_pause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    IS_PAUSED.update(deps.storage, |_| -> StdResult<bool> { Ok(true) })?;

    Ok(Response::new().add_attributes([("action", "try_pause")]))
}

pub fn try_unpause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrWorker,
    )?;

    IS_PAUSED.update(deps.storage, |_| -> StdResult<bool> { Ok(false) })?;

    Ok(Response::new().add_attributes([("action", "try_unpause")]))
}

pub fn try_set_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    owner_list: Vec<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    let collection_address = deps.api.addr_validate(&collection)?;
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
    collection_list: Vec<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    for collection_address in collection_list {
        let collection_address = deps.api.addr_validate(&collection_address)?;
        OWNERS.remove(deps.storage, &collection_address);
    }

    Ok(Response::new().add_attribute("action", "try_remove_collections"))
}

pub fn try_deposit_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;

    let collection_address = deps.api.addr_validate(&collection)?;
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
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    collection: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_withdraw_liquidity");
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    let owner_list = OWNERS.load(deps.storage, &collection_address)?;
    let outpost_list = vec![];
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
    let mut price_and_outpost_list =
        get_price_and_outpost_list(&deps.querier, &collection_address, &outpost_list)?;
    // sort descending by price
    price_and_outpost_list.sort_unstable_by_key(|(price, _)| Uint128::MAX - price);

    for (price, outpost) in price_and_outpost_list {
        // ignore outposts without offers
        if price.is_zero() {
            continue;
        }

        // let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        //     contract_addr: outpost.to_string(),
        //     msg: to_json_binary(
        //         &goplend_base::market_maker::outpost::msg::ExecuteMsg::RemoveOffer {
        //             collection_address: collection_address.to_string(),
        //             recipient: Some(sender_address.to_string()),
        //         },
        //     )?,
        //     funds: vec![],
        // });
        // response = response.add_message(msg);

        amount_undistributed += price;
        if amount_undistributed >= amount {
            amount_undistributed -= amount;
            break;
        }
    }

    if amount_undistributed < amount {
        Err(ContractError::ExceededAvailableAssetAmount)?;
    }

    UNDISTRIBUTED_LIQUIDITY.save(deps.storage, &collection_address, &amount_undistributed)?;

    Ok(response)
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection)?;
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

    // // we need to extract liquidity from offers starting from higher
    // let msg_list = query_collateral(deps.as_ref(), env, collection_address.to_string())?
    //     .into_iter()
    //     .filter(|x| !x.collection_info.token_id_list.is_empty())
    //     .map(|x| {
    //         Ok(CosmosMsg::Wasm(WasmMsg::Execute {
    //             contract_addr: x.outpost.to_string(),
    //             msg: to_json_binary(
    //                 &goplend_base::market_maker::outpost::msg::ExecuteMsg::WithdrawCollateral {
    //                     collection_address: collection_address.to_string(),
    //                     recipient: Some(sender_address.to_string()),
    //                 },
    //             )?,
    //             funds: vec![],
    //         }))
    //     })
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(Response::new()
        // .add_messages(msg_list)
        .add_attribute("action", "try_withdraw_collateral"))
}

pub fn try_update_offers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    from_to_price_list: Vec<(Uint128, Uint128)>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: config
                .controller
                .map(|x| x.into_iter().map(Some).collect())
                .unwrap_or_default(),
        },
    )?;

    let mut liquidity_to_add: Uint128 = Uint128::zero();
    let mut liquidity_to_sub: Uint128 = Uint128::zero();

    // let msg_list = from_to_price_and_outpost_list
    //     .iter()
    //     .map(|(price_before, price_after, outpost)| {
    //         liquidity_to_add += price_before;
    //         liquidity_to_sub += price_after;

    //         Ok(CosmosMsg::Wasm(WasmMsg::Execute {
    //             contract_addr: outpost.to_string(),
    //             msg: to_json_binary(
    //                 &goplend_base::market_maker::outpost::msg::ExecuteMsg::SetOffer {
    //                     collection_address: collection_address.to_string(),
    //                 },
    //             )?,
    //             funds: coins(price_after.u128(), config.token.try_get_native()?),
    //         }))
    //     })
    //     .collect::<StdResult<Vec<_>>>()?;

    // previous offers will be removed, new offers will be set
    UNDISTRIBUTED_LIQUIDITY.update(
        deps.storage,
        &collection_address,
        |x| -> StdResult<Uint128> {
            Ok(x.unwrap_or_default() + liquidity_to_add - liquidity_to_sub)
        },
    )?;

    Ok(Response::new()
        // .add_messages(msg_list)
        .add_attribute("action", "try_update_offers"))
}

/// user actions are disabled when the contract is paused
pub fn check_pause_state(storage: &dyn Storage) -> StdResult<()> {
    if IS_PAUSED.load(storage)? {
        Err(ContractError::ContractIsPaused)?;
    }

    Ok(())
}
