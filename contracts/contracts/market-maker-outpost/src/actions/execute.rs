use cosmwasm_std::{
    coin, coins, to_json_binary, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};

use stargaze_marketplace_v2::orders::OrderDetails;

use goplend_base::{
    error::ContractError,
    lending_platform::types::CollectionInfo,
    utils::{check_authorization, check_funds, unwrap_field, AuthType, FundsType},
};

use crate::helpers::{get_config, get_outpost_offer};

use super::query::{get_offer_price, query_collateral};

pub fn try_set_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_address: String,
) -> Result<Response, ContractError> {
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    let (config, hub) = get_config(deps.as_ref())?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![Some(hub.clone())],
        },
    )?;

    let marketplace = unwrap_field(config.marketplace, "marketplace")?;
    let offer_price = get_offer_price(
        &deps.querier,
        &marketplace,
        &collection_address,
        env.contract.address,
    );

    let mut msg_list = vec![
        // set new offer
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            msg: to_json_binary(
                &stargaze_marketplace_v2::msg::ExecuteMsg::SetCollectionBid {
                    collection: collection_address,
                    details: OrderDetails {
                        price: coin(asset_amount.u128(), asset_info.try_get_native()?),
                        finder: None,
                        recipient: None,
                    },
                },
            )?,
            funds: coins(asset_amount.u128(), asset_info.try_get_native()?),
        }),
    ];

    // return previous offer to hub
    if !offer_price.is_zero() {
        msg_list.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: hub.to_string(),
            amount: coins(offer_price.u128(), config.token.try_get_native()?),
        }));
    }

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_set_offer"))
}

pub fn try_remove_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_address: String,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let (config, hub) = get_config(deps.as_ref())?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![Some(hub.clone())],
        },
    )?;

    let marketplace = unwrap_field(config.marketplace, "marketplace")?;
    let offer = get_outpost_offer(
        &deps.querier,
        &marketplace,
        collection_address,
        env.contract.address,
    )?;

    let msg_list = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            msg: to_json_binary(
                &stargaze_marketplace_v2::msg::ExecuteMsg::RemoveCollectionBid { id: offer.id },
            )?,
            funds: vec![],
        }),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.unwrap_or(hub.to_string()),
            amount: vec![offer.details.price],
        }),
    ];

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_remove_offer"))
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_address: String,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let (config, hub) = get_config(deps.as_ref())?;
    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![Some(hub.clone())],
        },
    )?;

    let CollectionInfo { token_id_list, .. } =
        query_collateral(deps.as_ref(), env, collection_address.clone())?;

    let msg_list = token_id_list
        .iter()
        .map(|token_id| {
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection_address.clone(),
                msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                    recipient: recipient.as_ref().unwrap_or(&hub.to_string()).to_string(),
                    token_id: token_id.to_string(),
                })?,
                funds: vec![],
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_withdraw_collateral"))
}
