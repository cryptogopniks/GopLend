use cosmwasm_std::{
    coins, to_json_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage, Uint128, WasmMsg,
};

use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;

use goplend_base::{
    assets::TokenUnverified,
    error::ContractError,
    lending_platform::types::CollectionInfo,
    market_maker::{
        state::{
            CONFIG, IS_LOCKED, LIQUIDITY, OWNERS, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT,
        },
        types::{Config, LiquidityInfo, OffersResponse, TransferAdminState},
    },
    utils::{
        check_authorization, check_funds, get_collection_operator_approvals, get_transfer_msg,
        unwrap_field, AuthType, FundsType,
    },
};

use super::query::query_offers;

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
    marketplace: Option<String>,
    controller: Option<Vec<String>>,
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

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
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

    // TODO: do we need other conditions?
    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != config.token {
        Err(ContractError::WrongAssetType)?;
    }

    LIQUIDITY.update(
        deps.storage,
        &collection_address,
        |x| -> StdResult<LiquidityInfo> {
            let mut liquidity_info = x.unwrap_or(LiquidityInfo::new_empty(&collection_address));
            liquidity_info.amount_undistributed += asset_amount;

            Ok(liquidity_info)
        },
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
    let marketplace = unwrap_field(config.marketplace, "marketplace")?;
    let owner_list = OWNERS.load(deps.storage, &collection_address)?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: owner_list.into_iter().map(Some).collect(),
        },
    )?;

    let mut liquidity = LIQUIDITY.load(deps.storage, &collection_address)?;

    // we have enough undistributed liquidity
    if amount <= liquidity.amount_undistributed {
        liquidity.amount_undistributed -= amount;
        LIQUIDITY.save(deps.storage, &collection_address, &liquidity)?;

        return Ok(response.add_message(get_transfer_msg(&sender_address, amount, &config.token)?));
    }

    // we need to extract liquidity from offers starting from higher
    let OffersResponse { price_list, .. } =
        query_offers(deps.as_ref(), env, collection_address.to_string())?;

    for price in price_list {
        liquidity.amount_undistributed += price;
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            msg: to_json_binary(&sg_marketplace::msg::ExecuteMsg::RemoveCollectionBid {
                collection: collection_address.to_string(),
            })?,
            funds: vec![],
        });
        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "try_withdraw_collateral"))
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
