use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};

use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;

use goplend_base::{
    adapters::marketplace::stargaze::{
        state::{CONFIG, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
        types::{Config, TransferAdminState},
    },
    error::ContractError,
    lending_platform::{
        msg::QueryMsg as LendingPlatformQueryMsg,
        types::{BiddedCollateralItem, CommonConfig},
    },
    utils::{
        check_funds, get_collection_operator_approvals, get_transfer_msg, unwrap_field, AuthType,
        FundsType,
    },
};

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let TransferAdminState {
        new_admin,
        deadline,
    } = TRANSFER_ADMIN_STATE.load(deps.storage)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
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
    marketplace: Option<String>,
    infinity_pool: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;

    if let Some(x) = admin {
        check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;
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
        check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = lending_platform {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = marketplace {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.marketplace = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = infinity_pool {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.infinity_pool = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_accept_bids(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bidded_collateral_item_list: Vec<BiddedCollateralItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![CONFIG.load(deps.storage)?.lending_platform],
        },
    )?;

    let Config {
        lending_platform,
        marketplace,
        ..
    } = CONFIG.load(deps.storage)?;
    let lending_platform = &unwrap_field(lending_platform, "lending_platform")?;
    let marketplace = &unwrap_field(marketplace, "marketplace")?;

    // check operator and add approvals if it's required
    let mut collection_list: Vec<String> = bidded_collateral_item_list
        .iter()
        .map(|x| x.collection.to_string())
        .collect();
    collection_list.sort_unstable();
    collection_list.dedup();

    let mut msg_list = get_collection_operator_approvals(
        deps.querier,
        &collection_list,
        &env.contract.address,
        marketplace,
    )?;

    let mut liquidated_value = Uint128::zero();

    for BiddedCollateralItem {
        collection,
        token_id,
        liquidator,
        liquidation_price,
        ..
    } in bidded_collateral_item_list
    {
        liquidated_value += liquidation_price;

        // TODO: check if changing nft owner will not reset offers
        // transfer nfts from platform
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection.to_string(),
            msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_id.to_string(),
            })?,
            funds: vec![],
        });

        msg_list.push(msg);

        // add accept offer msgs
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            // msg: to_json_binary(&MarketplaceExecuteMsg::AcceptCollectionOffer {
            //     collection: collection.to_string(),
            //     token_id: token_id.to_string(),
            //     creator: liquidator.to_string(),
            //     order_options: None,
            // })?,
            msg: to_json_binary(&MarketplaceExecuteMsg::AcceptCollectionBid {
                collection: collection.to_string(),
                token_id: token_id.parse().map_err(|_| ContractError::CustomError {
                    val: format!("{} token_id parsing error", token_id),
                })?,
                bidder: liquidator.to_string(),
                finder: None,
            })?,
            funds: vec![],
        });

        msg_list.push(msg);
    }

    // send liquidated value to platform
    let CommonConfig { main_currency, .. } = deps.querier.query_wasm_smart(
        lending_platform,
        &LendingPlatformQueryMsg::QueryCommonConfig {},
    )?;

    msg_list.push(get_transfer_msg(
        &sender_address,
        liquidated_value,
        &main_currency.token,
    )?);

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_accept_bids"))
}

// TODO: refactoring
fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    let worker = unwrap_field(worker, "worker");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorker => {
            if !((sender == admin) || (worker.is_ok() && sender == worker?)) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::Specified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !is_included {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorkerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (worker.is_ok() && sender == worker?) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
    };

    Ok(())
}
