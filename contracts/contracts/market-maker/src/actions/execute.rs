use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
    Uint128, WasmMsg,
};

use goplend_base::{
    assets::TokenUnverified,
    error::ContractError,
    lending_platform::{
        state::TRANSFER_ADMIN_TIMEOUT,
        types::{BiddedCollateralItem, CollectionInfo, TransferAdminState},
    },
    market_maker::{
        state::{
            COLLATERALS, CONFIG, IS_PAUSED, OFFERS, OWNERS, TRANSFER_ADMIN_STATE,
            UNDISTRIBUTED_LIQUIDITY,
        },
        types::Config,
    },
    utils::{check_authorization, check_funds, get_transfer_msg, AuthType, FundsType},
};

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

pub fn try_remove_offers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender_address);
    let amount = OFFERS
        .load(deps.storage, &collection_address)
        .unwrap_or_default()
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur);

    if amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    OFFERS.remove(deps.storage, &collection_address);

    Ok(Response::new()
        .add_message(get_transfer_msg(&recipient, amount, &config.token)?)
        .add_attribute("action", "try_remove_offers"))
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

    Ok(Response::new().add_attribute("action", "try_pause"))
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

    Ok(Response::new().add_attribute("action", "try_unpause"))
}

pub fn try_set_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    owner: String,
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
    let owner = deps.api.addr_validate(&owner)?;

    OWNERS.save(deps.storage, &collection_address, &owner)?;

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
    let owner = OWNERS.load(deps.storage, &collection_address)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::AdminOrSpecified {
            allowlist: vec![Some(owner)],
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

pub fn try_withdraw_undistributed_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    let owner = OWNERS.load(deps.storage, &collection_address)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![Some(owner)],
        },
    )?;

    let amount = UNDISTRIBUTED_LIQUIDITY
        .load(deps.storage, &collection_address)
        .unwrap_or_default();

    if amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    UNDISTRIBUTED_LIQUIDITY.remove(deps.storage, &collection_address);

    Ok(Response::new()
        .add_message(get_transfer_msg(&sender_address, amount, &config.token)?)
        .add_attribute("action", "try_withdraw_undistributed_liquidity"))
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_withdraw_collateral");
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    let owner = OWNERS.load(deps.storage, &collection_address)?;

    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![Some(owner.clone())],
        },
    )?;

    let mut collaterals = COLLATERALS.load(deps.storage, &owner).unwrap_or_default();
    let token_id_list = collaterals
        .iter()
        .find(|x| x.collection_address == collection_address)
        .map(|x| x.token_id_list.clone())
        .unwrap_or_default();

    if token_id_list.is_empty() {
        Err(ContractError::CollateralIsNotFound)?;
    }

    for token_id in token_id_list {
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection.to_string(),
            msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                recipient: owner.to_string(),
                token_id,
            })?,
            funds: vec![],
        }));
    }

    collaterals.retain(|x| x.collection_address != collection_address);

    if collaterals.is_empty() {
        COLLATERALS.remove(deps.storage, &owner);
    } else {
        COLLATERALS.save(deps.storage, &owner, &collaterals)?;
    }

    Ok(response)
}

// use try_set_offers instead
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
    let mut offers = OFFERS
        .load(deps.storage, &collection_address)
        .unwrap_or_default();

    for (price_before, price_after) in from_to_price_list {
        liquidity_to_add += price_before;
        liquidity_to_sub += price_after;

        // replace price
        offers = offers
            .into_iter()
            .map(|x| if x == price_before { price_after } else { x })
            .collect();

        // add new price
        if price_before.is_zero() {
            offers.push(price_after);
        }
    }

    offers.sort_unstable_by(|a, b| b.cmp(a));
    OFFERS.save(deps.storage, &collection_address, &offers)?;

    // previous offers will be removed, new offers will be set
    UNDISTRIBUTED_LIQUIDITY.update(
        deps.storage,
        &collection_address,
        |x| -> StdResult<Uint128> {
            Ok(x.unwrap_or_default() + liquidity_to_add - liquidity_to_sub)
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_update_offers"))
}

pub fn try_set_offers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    price_list: Vec<Uint128>,
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

    let liquidity_to_add = OFFERS
        .load(deps.storage, &collection_address)
        .unwrap_or_default()
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur);
    let liquidity_to_sub = price_list
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur);

    OFFERS.save(deps.storage, &collection_address, &price_list)?;

    // previous offers will be removed, new offers will be set
    UNDISTRIBUTED_LIQUIDITY.update(
        deps.storage,
        &collection_address,
        |x| -> StdResult<Uint128> {
            Ok(x.unwrap_or_default() + liquidity_to_add - liquidity_to_sub)
        },
    )?;

    Ok(Response::new().add_attribute("action", "try_set_offers"))
}

pub fn try_accept_offers(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bidded_collateral_item_list: Vec<BiddedCollateralItem>,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_accept_offers");
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![config.lending_platform],
        },
    )?;

    let mut amount_to_send = Uint128::zero();

    for BiddedCollateralItem {
        collection,
        token_item,
        liquidation_price,
        ..
    } in bidded_collateral_item_list
    {
        // summarize executed offers to send to lending platform
        amount_to_send += liquidation_price;

        // remove executed offers
        OFFERS.update(deps.storage, &collection, |x| -> StdResult<Vec<Uint128>> {
            let mut offers = x.unwrap_or_default();
            offers.retain(|x| x != liquidation_price);
            Ok(offers)
        })?;

        // update liquidated collaterals
        let collection_owner = OWNERS.load(deps.storage, &collection)?;

        COLLATERALS.update(
            deps.storage,
            &collection_owner,
            |x| -> StdResult<Vec<CollectionInfo<Addr>>> {
                let mut collaterals: Vec<CollectionInfo<Addr>> = x.unwrap_or_default();

                if collaterals
                    .iter()
                    .all(|x| x.collection_address != collection)
                {
                    collaterals.push(CollectionInfo {
                        collection_address: collection.clone(),
                        token_id_list: vec![],
                    });
                }

                collaterals = collaterals
                    .into_iter()
                    .map(|mut x| {
                        if x.collection_address == collection {
                            x.token_id_list.push(token_item.id.clone());
                        }

                        x
                    })
                    .collect();

                Ok(collaterals)
            },
        )?;

        // transfer liquidated nfts from lending platform
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection.to_string(),
            msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_item.id,
            })?,
            funds: vec![],
        }));
    }

    Ok(response.add_message(get_transfer_msg(
        &sender_address,
        amount_to_send,
        &config.token,
    )?))
}

/// user actions are disabled when the contract is paused
fn check_pause_state(storage: &dyn Storage) -> StdResult<()> {
    if IS_PAUSED.load(storage)? {
        Err(ContractError::ContractIsPaused)?;
    }

    Ok(())
}
