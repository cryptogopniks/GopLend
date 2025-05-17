use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Uint128};
use cw_gopniks::{
    assets::{get_transfer_msg, Funds, InfoResp, TokenUnverified},
    auth::{Auth, TransferAdminState},
    nft,
};

use goplend_base::{
    error::ContractError,
    lending_platform::{
        state::TRANSFER_ADMIN_TIMEOUT,
        types::{BiddedCollateralItem, CollectionInfo},
    },
    market_maker::{
        state::{COLLATERALS, CONFIG, IS_PAUSED, OFFERS, OWNERS, UNDISTRIBUTED_LIQUIDITY},
        types::Config,
    },
};

pub fn try_accept_admin_role(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    TransferAdminState::accept_admin(&mut deps, &env, &sender)?;

    CONFIG.update(deps.storage, |mut x| -> StdResult<_> {
        x.admin = sender;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_accept_admin_role"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    controller: Option<Vec<String>>,
    lending_platform: Option<String>,
    oracle: Option<String>,
    token: Option<TokenUnverified>,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;

    if let Some(x) = admin {
        Auth::simple(current_admin).assert(&sender)?;
        TransferAdminState::update_admin(
            &mut deps,
            &env,
            &sender,
            &current_admin,
            &x,
            TRANSFER_ADMIN_TIMEOUT,
        )?;

        is_config_updated = true;
    }

    if let Some(x) = worker {
        Auth::simple(current_admin).assert(&sender)?;
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = controller {
        Auth::simple(current_admin).assert(&sender)?;
        config.controller = Some(
            x.iter()
                .map(|y| deps.api.addr_validate(y))
                .collect::<StdResult<Vec<_>>>()?,
        );
        is_config_updated = true;
    }

    if let Some(x) = lending_platform {
        Auth::simple(current_admin).assert(&sender)?;
        config.lending_platform = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = oracle {
        Auth::simple(current_admin).assert(&sender)?;
        config.oracle = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = token {
        Auth::simple(current_admin).assert(&sender)?;
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
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    Auth::simple(&config.admin).assert(&sender)?;

    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender);
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
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    Auth::simple_optional(deps.api, &admin, &worker)?.assert(&sender)?;

    IS_PAUSED.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "try_pause"))
}

pub fn try_unpause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    Auth::simple_optional(deps.api, &admin, &worker)?.assert(&sender)?;

    IS_PAUSED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "try_unpause"))
}

pub fn try_set_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    owner: String,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;
    Auth::simple(&config.admin).assert(&sender)?;

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
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;
    Auth::simple(&config.admin).assert(&sender)?;

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
    let InfoResp {
        sender,
        asset_amount,
        asset_token,
    } = Funds::single(None, None).check(deps.api, &info)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    let owner = OWNERS.load(deps.storage, &collection_address)?;
    Auth::simple_specified(deps.api, &config.admin, &[&owner])?.assert(&sender)?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_token != config.token {
        Err(ContractError::WrongAssetType)?;
    }

    UNDISTRIBUTED_LIQUIDITY.update(deps.storage, &collection_address, |x| -> StdResult<_> {
        Ok(x.unwrap_or_default() + asset_amount)
    })?;

    Ok(Response::new().add_attribute("action", "try_deposit_liquidity"))
}

pub fn try_withdraw_undistributed_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    let owner = OWNERS.load(deps.storage, &collection_address)?;
    Auth::specified(deps.api, &[&owner])?.assert(&sender)?;

    let amount = UNDISTRIBUTED_LIQUIDITY
        .load(deps.storage, &collection_address)
        .unwrap_or_default();

    if amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    UNDISTRIBUTED_LIQUIDITY.remove(deps.storage, &collection_address);

    Ok(Response::new()
        .add_message(get_transfer_msg(&sender, amount, &config.token)?)
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
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let owner = OWNERS.load(deps.storage, &collection_address)?;
    Auth::specified(deps.api, &[&owner])?.assert(&sender)?;

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
        response = response.add_message(nft::get_transfer_msg(&collection, &owner, token_id)?);
    }

    collaterals.retain(|x| x.collection_address != collection_address);

    if collaterals.is_empty() {
        COLLATERALS.remove(deps.storage, &owner);
    } else {
        COLLATERALS.save(deps.storage, &owner, &collaterals)?;
    }

    Ok(response)
}

pub fn try_set_offers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    price_list: Vec<Uint128>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let collection_address = deps.api.addr_validate(&collection)?;
    let config = CONFIG.load(deps.storage)?;
    Auth::specified(deps.api, &config.controller.unwrap_or_default())?.assert(&sender)?;

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
    UNDISTRIBUTED_LIQUIDITY.update(deps.storage, &collection_address, |x| -> StdResult<_> {
        Ok(x.unwrap_or_default() + liquidity_to_add - liquidity_to_sub)
    })?;

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
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;
    Auth::specified(deps.api, &[&config.lending_platform.unwrap()])?.assert(&sender)?;

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
        OFFERS.update(deps.storage, &collection, |x| -> StdResult<_> {
            let mut offers = x.unwrap_or_default();
            offers.retain(|x| x != liquidation_price);
            Ok(offers)
        })?;

        // update liquidated collaterals
        let collection_owner = OWNERS.load(deps.storage, &collection)?;

        COLLATERALS.update(deps.storage, &collection_owner, |x| -> StdResult<_> {
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
        })?;

        // transfer liquidated nfts from lending platform
        response = response.add_message(nft::get_transfer_msg(
            &collection,
            &env.contract.address,
            token_item.id,
        )?);
    }

    Ok(response.add_message(get_transfer_msg(&sender, amount_to_send, &config.token)?))
}

/// user actions are disabled when the contract is paused
fn check_pause_state(storage: &dyn Storage) -> StdResult<()> {
    if IS_PAUSED.load(storage)? {
        Err(ContractError::ContractIsPaused)?;
    }

    Ok(())
}
