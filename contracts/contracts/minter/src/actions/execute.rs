use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
};

use osmosis_std::types::{
    cosmos::bank::v1beta1::{DenomUnit as OsmosisDenomUnit, Metadata as OsmosisMetadata},
    osmosis::tokenfactory::v1beta1 as OsmosisFactory,
};

use goplend_base::{
    assets::{Currency, Token, TokenUnverified},
    error::ContractError,
    minter::{
        state::{
            CONFIG, DEFAULT_DECIMALS, OWNERS, OWNER_AND_DECIMALS, SAVE_CW20_ADDRESS_REPLY,
            TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT,
        },
        types::{Config, DenomUnit, Metadata, TransferAdminState},
    },
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

pub fn try_create_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_owner: String,
    subdenom: String,
    decimals: Option<u8>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let owner = deps.api.addr_validate(&token_owner)?;
    let creator = env.contract.address;

    let full_denom = &get_full_denom(&creator, &subdenom);
    let currency = Currency::new(
        &Token::new_native(full_denom),
        decimals.unwrap_or(DEFAULT_DECIMALS),
    );

    OWNERS.update(
        deps.storage,
        full_denom,
        |x| -> StdResult<(Currency<Token>, Addr)> {
            match x {
                Some(_) => Err(ContractError::DenomExists)?,
                None => Ok((currency, owner)),
            }
        },
    )?;

    let msg: CosmosMsg = OsmosisFactory::MsgCreateDenom {
        sender: creator.to_string(),
        subdenom,
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_create_native")]))
}

#[allow(clippy::too_many_arguments)]
pub fn try_create_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_owner: String,
    name: String,
    symbol: String,
    decimals: Option<u8>,
    marketing: Option<cw20_base::msg::InstantiateMarketingInfo>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let owner = deps.api.addr_validate(&token_owner)?;
    let creator = env.contract.address;
    let decimals = decimals.unwrap_or(DEFAULT_DECIMALS);

    OWNER_AND_DECIMALS.push_back(deps.storage, &(owner, decimals))?;

    let cw20_msg = cw20_base::msg::InstantiateMsg {
        name,
        symbol: symbol.to_string(),
        decimals,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: creator.to_string(),
            cap: None,
        }),
        marketing,
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(creator.to_string()),
        code_id: unwrap_field(
            CONFIG.load(deps.storage)?.cw20_base_code_id,
            "cw20_base_code_id",
        )?,
        label: format!("GopLend {}", symbol),
        msg: to_json_binary(&cw20_msg)?,
        funds: vec![],
    });

    let submsg = SubMsg::reply_on_success(msg, SAVE_CW20_ADDRESS_REPLY);

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attributes([("action", "try_create_cw20")]))
}

pub fn try_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: TokenUnverified,
    amount: Uint128,
    recipient: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_mint");
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    deps.api.addr_validate(&recipient)?;
    let creator = &env.contract.address;
    let token = token.verify(deps.api)?;
    let token_denom = match token.clone() {
        Token::Native { denom } => denom,
        Token::Cw20 { address } => address.to_string(),
    };
    let (_, token_owner) = OWNERS
        .load(deps.storage, &token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    if token.is_native() {
        let amount = coin(amount.u128(), token_denom);

        let msg_list = vec![
            OsmosisFactory::MsgMint {
                sender: creator.to_string(),
                amount: Some(amount.clone().into()),
                mint_to_address: creator.to_string(),
            }
            .into(),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![amount],
            }),
        ];

        response = response.add_messages(msg_list);
    } else {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_denom,
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint { recipient, amount })?,
            funds: vec![],
        });

        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_burn");
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;

    let creator = &env.contract.address;
    let token_denom = match asset_info.clone() {
        Token::Native { denom } => denom,
        Token::Cw20 { address } => address.to_string(),
    };
    let (_, token_owner) = OWNERS
        .load(deps.storage, &token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    if asset_info.is_native() {
        let amount = coin(asset_amount.u128(), token_denom);

        let msg: CosmosMsg = OsmosisFactory::MsgBurn {
            sender: creator.to_string(),
            amount: Some(amount.into()),
            burn_from_address: creator.to_string(),
        }
        .into();

        response = response.add_message(msg);
    } else {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_denom,
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Burn {
                amount: asset_amount,
            })?,
            funds: vec![],
        });

        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_set_metadata_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: TokenUnverified,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let token = token.verify(deps.api)?;
    let token_denom = token.try_get_native()?;
    let (_, token_owner) = OWNERS
        .load(deps.storage, &token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::AdminOrWorkerOrSpecified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    let sender = env.contract.address.to_string();
    let Metadata {
        description,
        denom_units,
        base,
        display,
        name,
        symbol,
        ..
    } = metadata;

    let msg: CosmosMsg = OsmosisFactory::MsgSetDenomMetadata {
        sender,
        metadata: Some(OsmosisMetadata {
            description,
            denom_units: denom_units
                .into_iter()
                .map(
                    |DenomUnit {
                         denom,
                         exponent,
                         aliases,
                     }| OsmosisDenomUnit {
                        denom,
                        exponent,
                        aliases,
                    },
                )
                .collect(),
            base,
            display,
            name,
            symbol,
        }),
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_set_metadata_native")]))
}

pub fn try_update_marketing_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: TokenUnverified,
    project: Option<String>,
    description: Option<String>,
    marketing: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let token = token.verify(deps.api)?;
    let token_denom = &token.try_get_cw20()?.to_string();
    let (_, token_owner) = OWNERS
        .load(deps.storage, token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::AdminOrWorkerOrSpecified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_denom.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_update_marketing_cw20")]))
}

pub fn try_upload_logo_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: TokenUnverified,
    logo: cw20::Logo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let token = token.verify(deps.api)?;
    let token_denom = &token.try_get_cw20()?.to_string();
    let (_, token_owner) = OWNERS
        .load(deps.storage, token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::AdminOrWorkerOrSpecified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_denom.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::UploadLogo(logo))?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_upload_logo_cw20")]))
}

pub fn try_change_admin_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: TokenUnverified,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let token = token.verify(deps.api)?;
    let token_denom = token.try_get_native()?;
    let (_, token_owner) = OWNERS
        .load(deps.storage, &token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::AdminOrWorkerOrSpecified {
            allowlist: vec![Some(token_owner.clone())],
        },
    )?;

    let msg: CosmosMsg = OsmosisFactory::MsgChangeAdmin {
        denom: token_denom,
        sender: env.contract.address.to_string(),
        new_admin: token_owner.to_string(),
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_change_admin_native")]))
}

pub fn try_update_minter_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: TokenUnverified,
    new_minter: String,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let token = token.verify(deps.api)?;
    let token_denom = &token.try_get_cw20()?.to_string();
    let (_, token_owner) = OWNERS
        .load(deps.storage, token_denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::AdminOrWorkerOrSpecified {
            allowlist: vec![Some(token_owner)],
        },
    )?;

    deps.api.addr_validate(&new_minter)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_denom.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::UpdateMinter {
            new_minter: Some(new_minter),
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes([("action", "try_update_minter_cw20")]))
}

pub fn try_register_currency(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    currency: Currency<TokenUnverified>,
    creator: String,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let creator = deps.api.addr_validate(&creator)?;
    let token = currency.token.verify(deps.api)?;
    let currency = Currency::new(&token, currency.decimals);
    let token_denom = match token {
        Token::Native { denom } => denom,
        Token::Cw20 { address } => address.to_string(),
    };

    OWNERS.update(
        deps.storage,
        &token_denom,
        |x| -> StdResult<(Currency<Token>, Addr)> {
            match x {
                Some(_) => Err(ContractError::DenomExists)?,
                None => Ok((currency, creator)),
            }
        },
    )?;

    Ok(Response::new().add_attributes([("action", "try_register_currency")]))
}

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

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    cw20_base_code_id: Option<u64>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    let mut response = Response::new().add_attribute("action", "try_update_config");
    let mut config = CONFIG.load(deps.storage)?;

    if let Some(x) = admin {
        let block_time = env.block.time.seconds();
        let new_admin = &deps.api.addr_validate(&x)?;

        TRANSFER_ADMIN_STATE.save(
            deps.storage,
            &TransferAdminState {
                new_admin: new_admin.to_owned(),
                deadline: block_time + TRANSFER_ADMIN_TIMEOUT,
            },
        )?;

        response = response.add_attribute("admin", new_admin);
    }

    if let Some(x) = worker {
        config.worker = Some(deps.api.addr_validate(&x)?);
        response = response.add_attribute("worker", x);
    }

    if let Some(x) = cw20_base_code_id {
        config.cw20_base_code_id = Some(x);
        response = response.add_attribute("cw20_base_code_id", x.to_string());
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response)
}

pub fn save_cw20_address(
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

    let cw20_address = &unwrap_field(
        instantiate_event
            .attributes
            .iter()
            .find(|x| x.key == "_contract_address"),
        "cw20_address",
    )?
    .value;

    let (owner, decimals) = unwrap_field(
        OWNER_AND_DECIMALS.pop_front(deps.storage)?,
        "owner_and_decimals",
    )?;

    let currency = Currency::new(
        &Token::new_cw20(&deps.api.addr_validate(cw20_address)?),
        decimals,
    );

    OWNERS.update(
        deps.storage,
        cw20_address,
        |x| -> StdResult<(Currency<Token>, Addr)> {
            match x {
                Some(_) => Err(ContractError::DenomExists)?,
                None => Ok((currency, owner)),
            }
        },
    )?;

    Ok(Response::new().add_attributes([("cw20_address", cw20_address)]))
}

pub fn try_update_token_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
    owner: String,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;

    let owner = deps.api.addr_validate(&owner)?;
    let (currency, _) = OWNERS.load(deps.storage, &denom)?;
    OWNERS.save(deps.storage, &denom, &(currency, owner))?;

    Ok(Response::new().add_attributes([("action", "try_update_token_owner")]))
}

fn get_full_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{creator}/{subdenom}")
}

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
