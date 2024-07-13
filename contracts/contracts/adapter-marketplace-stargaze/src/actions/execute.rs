use cosmwasm_std::{
    coins, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, WasmMsg,
};

use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        marketplace::stargaze::{
            state::{CONFIG, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT},
            types::{Config, MarketplaceVersion, TransferAdminState},
        },
    },
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::types::{BidType, BiddedCollateralItem, CommonConfig},
    oracle::types::PriceRatioItem,
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
    lending_platform_main: Option<String>,
    lending_platform_stable: Option<String>,
    oracle: Option<String>,
    marketplace: Option<String>,
    adapter_dex: Option<String>,
    marketplace_version: Option<MarketplaceVersion>,
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

    if let Some(x) = lending_platform_main {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform_main = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = lending_platform_stable {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform_stable = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = oracle {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.oracle = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = marketplace {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.marketplace = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = adapter_dex {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.adapter_dex = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = marketplace_version {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.marketplace_version = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

// TODO: support different marketplace versions
pub fn try_accept_offers(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bidded_collateral_item_list: Vec<BiddedCollateralItem>,
    route: Route,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let Config {
        lending_platform_main,
        lending_platform_stable,
        marketplace,
        oracle,
        adapter_dex,
        ..
    } = CONFIG.load(deps.storage)?;
    let marketplace = &unwrap_field(marketplace, "marketplace")?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![lending_platform_main.clone(), lending_platform_stable],
        },
    )?;

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
        token_item,
        liquidator,
        liquidation_price,
        collateral_price,
        bid_type,
        ..
    } in bidded_collateral_item_list
    {
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
            BidType::CollectionOffer => MarketplaceExecuteMsg::AcceptCollectionBid {
                collection: collection.to_string(),
                token_id: token_item
                    .id
                    .parse()
                    .map_err(|_| ContractError::CustomError {
                        val: format!("{} token_id parsing error", token_item.id),
                    })?,
                bidder: liquidator.to_string(),
                finder: None,
                amount: collateral_price,
            },
            BidType::TokenOffer => MarketplaceExecuteMsg::AcceptBid {
                collection: collection.to_string(),
                token_id: token_item
                    .id
                    .parse()
                    .map_err(|_| ContractError::CustomError {
                        val: format!("{} token_id parsing error", token_item.id),
                    })?,
                bidder: liquidator.to_string(),
                finder: None,
                amount: collateral_price,
            },
            _ => unimplemented!(),
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            msg: to_json_binary(&accept_offer_msg)?,
            funds: vec![],
        });

        msg_list.push(msg);
    }

    // query main currency
    let CommonConfig { main_currency, .. } = deps.querier.query_wasm_smart(
        unwrap_field(lending_platform_main, "lending_platform_main")?,
        &goplend_base::lending_platform::msg::QueryMsg::QueryCommonConfig {},
    )?;

    match route {
        Route::Empty => {
            // send liquidated value to platform
            msg_list.push(get_transfer_msg(
                &sender_address,
                liquidated_value,
                &main_currency.token,
            )?);
        }
        _ => {
            let oracle = &unwrap_field(oracle, "oracle")?;
            let adapter_dex = &unwrap_field(adapter_dex, "adapter_dex")?;

            // query price ratio
            let PriceRatioItem {
                value: price_ratio,
                is_outdated,
                ..
            } = deps.querier.query_wasm_smart(
                oracle,
                &goplend_base::oracle::msg::QueryMsg::QueryPriceRatio {},
            )?;

            if is_outdated {
                Err(ContractError::OutdatedPrice {
                    val: price_ratio.to_string(),
                })?;
            }

            let liquidated_value_in_main_currency =
                (u128_to_dec(liquidated_value) / price_ratio).to_uint_floor();

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: adapter_dex.to_string(),
                msg: to_json_binary(
                    &goplend_base::adapters::dex::stargaze::msg::ExecuteMsg::Swap {
                        route,
                        price_ratio,
                    },
                )?,
                funds: coins(
                    liquidated_value_in_main_currency.u128(),
                    main_currency.token.try_get_native()?,
                ),
            });

            msg_list.push(msg);
        }
    }

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
