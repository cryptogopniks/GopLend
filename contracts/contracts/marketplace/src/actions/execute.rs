use cosmwasm_std::{
    coin, to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};

use stargaze_marketplace_v2::orders::OrderDetails;

use goplend_base::{
    error::ContractError,
    lending_platform::types::{BidType, BiddedCollateralItem, CommonConfig},
    marketplace::{
        state::{CONFIG, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, TransferAdminState},
    },
    utils::{
        check_authorization, check_funds, get_collection_operator_approvals, get_transfer_msg,
        AuthType, FundsType,
    },
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
    lending_platform: Option<String>,
    oracle: Option<String>,
    marketplace: Option<String>,
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

    if let Some(x) = lending_platform {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.lending_platform = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    if let Some(x) = oracle {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.oracle = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    if let Some(x) = marketplace {
        check_authorization(
            &sender_address,
            current_admin,
            current_worker,
            AuthType::Admin,
        )?;
        config.marketplace = deps.api.addr_validate(&x)?;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_accept_offers(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bidded_collateral_item_list: Vec<BiddedCollateralItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let Config {
        admin,
        worker,
        lending_platform,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;

    check_authorization(
        &sender_address,
        &admin,
        &worker,
        AuthType::Specified {
            allowlist: vec![Some(lending_platform.clone())],
        },
    )?;

    // check operator and add approvals if it's required
    let mut collection_list: Vec<String> = bidded_collateral_item_list
        .iter()
        .map(|x| x.collection.to_string())
        .collect();
    collection_list.sort_unstable();
    collection_list.dedup();

    // query main currency
    let CommonConfig { main_currency, .. } = deps.querier.query_wasm_smart(
        lending_platform,
        &goplend_base::lending_platform::msg::QueryMsg::QueryCommonConfig {},
    )?;

    let mut msg_list = get_collection_operator_approvals(
        deps.querier,
        &collection_list,
        &env.contract.address,
        marketplace.clone(),
    )?;

    let mut liquidated_value = Uint128::zero();

    for bidded_collateral_item in bidded_collateral_item_list {
        let BiddedCollateralItem {
            collection,
            token_item,
            liquidation_price,
            bid_amount,
            bid_type,
            ..
        } = bidded_collateral_item;

        liquidated_value += liquidation_price;

        // transfer nfts from platform
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection.to_string(),
            msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_item.id.to_string(),
            })?,
            funds: vec![],
        });

        msg_list.push(msg);

        // add accept offer msgs
        let accept_offer_msg = match bid_type {
            BidType::CollectionOffer => {
                stargaze_marketplace_v2::msg::ExecuteMsg::AcceptCollectionBid {
                    id: "".to_string(),
                    token_id: token_item
                        .id
                        .parse()
                        .map_err(|_| ContractError::CustomError {
                            val: format!("{} token_id parsing error", token_item.id),
                        })?,
                    details: OrderDetails {
                        price: coin(bid_amount.u128(), main_currency.token.try_get_native()?),
                        finder: None,
                        recipient: None,
                    },
                }
            }
            _ => unimplemented!(),
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            msg: to_json_binary(&accept_offer_msg)?,
            funds: vec![],
        });

        msg_list.push(msg);
    }

    // send liquidated value to platform
    msg_list.push(get_transfer_msg(
        &sender_address,
        liquidated_value,
        &main_currency.token,
    )?);

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_accept_bids"))
}
