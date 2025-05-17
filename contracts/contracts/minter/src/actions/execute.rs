use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage, SubMsg, SubMsgResult, Uint128, WasmMsg,
};
use cw_gopniks::{
    assets::{get_upload_logo_msg, Currency, Funds, InfoResp, Logo, Token, TokenUnverified},
    auth::{Auth, AuthError, TransferAdminState},
};

use osmosis_std::types::{
    cosmos::bank::v1beta1::{DenomUnit as OsmosisDenomUnit, Metadata as OsmosisMetadata},
    osmosis::tokenfactory::v1beta1 as OsmosisFactory,
};

use goplend_base::{
    error::ContractError,
    lending_platform::state::TRANSFER_ADMIN_TIMEOUT,
    minter::{
        state::{
            CONFIG, CURRENCIES, DEFAULT_DECIMALS, FAUCET_CONFIG, IS_PAUSED, LAST_CLAIM_DATE,
            SAVE_CW20_ADDRESS_REPLY, TEMPORARY_CURRENCY, TOKEN_COUNT, TRANSFER_OWNER_STATE,
        },
        types::{Config, CurrencyInfo, DenomUnit, FaucetConfig, Metadata},
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

pub fn try_accept_token_owner_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let block_time = env.block.time.seconds();
    let config = CONFIG.load(deps.storage)?;
    let mut transfer_owner_state_list = TRANSFER_OWNER_STATE.load(deps.storage).unwrap_or_default();

    let current_transfer_owner_state_list = transfer_owner_state_list
        .iter()
        .filter(|(_, transfer_state)| transfer_state.get_new_admin() == sender)
        .collect::<Vec<_>>();

    if current_transfer_owner_state_list.is_empty() {
        Err(ContractError::Auth(AuthError::Unauthorized))?;
    }

    for (denom_or_address, transfer_admin_state) in current_transfer_owner_state_list {
        if block_time >= transfer_admin_state.get_deadline() {
            Err(ContractError::Auth(AuthError::TransferAdminDeadline))?;
        }

        let new_token_owner = &transfer_admin_state.get_new_admin();
        let mut currency_info = CURRENCIES.load(deps.storage, denom_or_address)?;
        let current_owner_token_count = TOKEN_COUNT.load(deps.storage, &currency_info.owner)? - 1;
        let new_owner_token_count = TOKEN_COUNT
            .load(deps.storage, new_token_owner)
            .unwrap_or_default()
            + 1;

        if !config.whitelist.contains(new_token_owner)
            && new_owner_token_count > config.max_tokens_per_owner
        {
            Err(ContractError::TokenLimit)?;
        }

        if current_owner_token_count == 0 {
            TOKEN_COUNT.remove(deps.storage, &currency_info.owner);
        } else {
            TOKEN_COUNT.save(
                deps.storage,
                &currency_info.owner,
                &current_owner_token_count,
            )?;
        }
        TOKEN_COUNT.save(deps.storage, new_token_owner, &new_owner_token_count)?;

        currency_info.owner = new_token_owner.to_owned();
        CURRENCIES.save(deps.storage, denom_or_address, &currency_info)?;
    }

    transfer_owner_state_list
        .retain(|(_, transfer_state)| transfer_state.get_new_admin() != sender);
    TRANSFER_OWNER_STATE.save(deps.storage, &transfer_owner_state_list)?;

    Ok(Response::new().add_attribute("action", "try_accept_token_owner_role"))
}

pub fn try_pause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let Config { admin, .. } = CONFIG.load(deps.storage)?;
    Auth::simple(&admin).assert(&sender)?;

    IS_PAUSED.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "try_pause"))
}

pub fn try_unpause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let Config { admin, .. } = CONFIG.load(deps.storage)?;
    Auth::simple(&admin).assert(&sender)?;

    IS_PAUSED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "try_unpause"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    whitelist: Option<Vec<String>>,
    cw20_code_id: Option<u64>,
    permissionless_token_creation: Option<bool>,
    permissionless_token_registration: Option<bool>,
    max_tokens_per_owner: Option<u16>,
) -> Result<Response, ContractError> {
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;
    let current_admin = &config.admin;
    Auth::simple(current_admin).assert(&sender)?;

    if let Some(x) = admin {
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

    if let Some(x) = whitelist {
        config.whitelist = x
            .iter()
            .map(|x| deps.api.addr_validate(x))
            .collect::<StdResult<Vec<Addr>>>()?;
        is_config_updated = true;
    }

    if let Some(x) = cw20_code_id {
        config.cw20_code_id = Some(x);
        is_config_updated = true;
    }

    if let Some(x) = permissionless_token_creation {
        config.permissionless_token_creation = x;
        is_config_updated = true;
    }

    if let Some(x) = permissionless_token_registration {
        config.permissionless_token_registration = x;
        is_config_updated = true;
    }

    if let Some(x) = max_tokens_per_owner {
        config.max_tokens_per_owner = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_create_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    whitelist: Option<Vec<String>>,
    permissionless_burning: Option<bool>,
    subdenom: String,
    decimals: Option<u8>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::single(None, None).check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;

    if !config.permissionless_token_creation && !config.whitelist.contains(&sender) {
        Err(ContractError::Auth(AuthError::Unauthorized))?;
    }

    let owner = owner
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender.clone());
    let creator = env.contract.address;
    let full_denom = &get_full_denom(&creator, &subdenom);
    let decimals = decimals.unwrap_or(DEFAULT_DECIMALS);
    let currency = Currency::new(&Token::new_native(full_denom), decimals);
    let whitelist = whitelist
        .unwrap_or(vec![sender.to_string()])
        .iter()
        .map(|x| deps.api.addr_validate(x))
        .collect::<StdResult<Vec<Addr>>>()?;

    CURRENCIES.update(deps.storage, full_denom, |x| -> StdResult<CurrencyInfo> {
        match x {
            Some(_) => Err(ContractError::DenomExists)?,
            None => Ok(CurrencyInfo {
                currency,
                cw20_code_id: None,
                owner,
                whitelist,
                permissionless_burning: permissionless_burning.unwrap_or_default(),
            }),
        }
    })?;

    TOKEN_COUNT.update(deps.storage, &sender, |x| -> StdResult<u16> {
        let token_count = x.unwrap_or_default() + 1;

        if !config.whitelist.contains(&sender) && token_count > config.max_tokens_per_owner {
            Err(ContractError::TokenLimit)?;
        }

        Ok(token_count)
    })?;

    let msg: CosmosMsg = OsmosisFactory::MsgCreateDenom {
        sender: creator.to_string(),
        subdenom,
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_create_native"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_create_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    whitelist: Option<Vec<String>>,
    permissionless_burning: Option<bool>,
    cw20_code_id: Option<u64>,
    name: String,
    symbol: String,
    decimals: Option<u8>,
    marketing: Option<cw20_base::msg::InstantiateMarketingInfo>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;

    if !config.permissionless_token_creation && !config.whitelist.contains(&sender) {
        Err(ContractError::Auth(AuthError::Unauthorized))?;
    }

    let owner = owner
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender.clone());
    let creator = env.contract.address;
    let decimals = decimals.unwrap_or(DEFAULT_DECIMALS);
    let currency = Currency::new(&Token::new_cw20(&sender), decimals);
    let cw20_code_id = cw20_code_id.map_or(config.cw20_code_id, Some).unwrap();
    let whitelist = whitelist
        .unwrap_or(vec![sender.to_string()])
        .iter()
        .map(|x| deps.api.addr_validate(x))
        .collect::<StdResult<Vec<Addr>>>()?;
    let mut marketing_info = marketing.unwrap_or(cw20_base::msg::InstantiateMarketingInfo {
        project: None,
        description: None,
        marketing: None,
        logo: None,
    });
    marketing_info.marketing = Some(creator.to_string());

    TOKEN_COUNT.update(deps.storage, &sender, |x| -> StdResult<u16> {
        let token_count = x.unwrap_or_default() + 1;

        if !config.whitelist.contains(&sender) && token_count > config.max_tokens_per_owner {
            Err(ContractError::TokenLimit)?;
        }

        Ok(token_count)
    })?;

    TEMPORARY_CURRENCY.push_back(
        deps.storage,
        &CurrencyInfo {
            currency, // will be rewritten later
            cw20_code_id: Some(cw20_code_id),
            owner,
            whitelist,
            permissionless_burning: permissionless_burning.unwrap_or_default(),
        },
    )?;

    let cw20_msg = cw20_base::msg::InstantiateMsg {
        name,
        symbol: symbol.to_string(),
        decimals,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: creator.to_string(),
            cap: None,
        }),
        marketing: Some(marketing_info),
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(config.admin.to_string()), // to allow migrate special cw20 based contracts
        code_id: cw20_code_id,
        label: format!("CW20 based token {}", symbol),
        msg: to_json_binary(&cw20_msg)?,
        funds: vec![],
    });

    let submsg = SubMsg::reply_on_success(msg, SAVE_CW20_ADDRESS_REPLY);

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("action", "try_create_cw20"))
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

    let instantiate_event = res.events.iter().find(|x| x.ty == "instantiate").unwrap();
    let cw20_address = &instantiate_event
        .attributes
        .iter()
        .find(|x| x.key == "_contract_address")
        .unwrap()
        .value;

    let mut currency_info = TEMPORARY_CURRENCY.pop_front(deps.storage)?.unwrap();
    currency_info.currency = Currency::new(
        &Token::new_cw20(&deps.api.addr_validate(cw20_address)?),
        currency_info.currency.decimals,
    );

    CURRENCIES.update(deps.storage, cw20_address, |x| -> StdResult<CurrencyInfo> {
        match x {
            Some(_) => Err(ContractError::DenomExists)?,
            None => Ok(currency_info),
        }
    })?;

    Ok(Response::new().add_attribute("cw20_address", cw20_address))
}

/// current token admin must send MsgChangeAdmin before/after this action to complete registration
#[allow(clippy::too_many_arguments)]
pub fn try_register_native(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
    owner: Option<String>,
    whitelist: Option<Vec<String>>,
    permissionless_burning: Option<bool>,
    decimals: Option<u8>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;

    if !config.permissionless_token_registration && !config.whitelist.contains(&sender) {
        Err(ContractError::Auth(AuthError::Unauthorized))?;
    }

    let owner = owner
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender.clone());
    let decimals = decimals.unwrap_or(DEFAULT_DECIMALS);
    let currency = Currency::new(&Token::new_native(&denom), decimals);
    let whitelist = whitelist
        .unwrap_or(vec![sender.to_string()])
        .iter()
        .map(|x| deps.api.addr_validate(x))
        .collect::<StdResult<Vec<Addr>>>()?;

    CURRENCIES.update(deps.storage, &denom, |x| -> StdResult<CurrencyInfo> {
        match x {
            Some(_) => Err(ContractError::DenomExists)?,
            None => Ok(CurrencyInfo {
                currency,
                cw20_code_id: None,
                owner,
                whitelist,
                permissionless_burning: permissionless_burning.unwrap_or_default(),
            }),
        }
    })?;

    TOKEN_COUNT.update(deps.storage, &sender, |x| -> StdResult<u16> {
        let token_count = x.unwrap_or_default() + 1;

        if !config.whitelist.contains(&sender) && token_count > config.max_tokens_per_owner {
            Err(ContractError::TokenLimit)?;
        }

        Ok(token_count)
    })?;

    Ok(Response::new().add_attribute("action", "try_register_currency"))
}

/// current token admin must update contract admin, minter, marketing before/after this action  \
/// to complete registration
#[allow(clippy::too_many_arguments)]
pub fn try_register_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    owner: Option<String>,
    whitelist: Option<Vec<String>>,
    permissionless_burning: Option<bool>,
    cw20_code_id: Option<u64>,
    decimals: Option<u8>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let config = CONFIG.load(deps.storage)?;

    if !config.permissionless_token_registration && !config.whitelist.contains(&sender) {
        Err(ContractError::Auth(AuthError::Unauthorized))?;
    }

    let owner = owner
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender.clone());
    let decimals = decimals.unwrap_or(DEFAULT_DECIMALS);
    let currency = Currency::new(
        &TokenUnverified::new_cw20(&address).verify(deps.api)?,
        decimals,
    );
    let cw20_code_id = cw20_code_id.map_or(config.cw20_code_id, Some).unwrap();
    let whitelist = whitelist
        .unwrap_or(vec![sender.to_string()])
        .iter()
        .map(|x| deps.api.addr_validate(x))
        .collect::<StdResult<Vec<Addr>>>()?;

    CURRENCIES.update(deps.storage, &address, |x| -> StdResult<CurrencyInfo> {
        match x {
            Some(_) => Err(ContractError::DenomExists)?,
            None => Ok(CurrencyInfo {
                currency,
                cw20_code_id: Some(cw20_code_id),
                owner,
                whitelist,
                permissionless_burning: permissionless_burning.unwrap_or_default(),
            }),
        }
    })?;

    TOKEN_COUNT.update(deps.storage, &sender, |x| -> StdResult<u16> {
        let token_count = x.unwrap_or_default() + 1;

        if !config.whitelist.contains(&sender) && token_count > config.max_tokens_per_owner {
            Err(ContractError::TokenLimit)?;
        }

        Ok(token_count)
    })?;

    Ok(Response::new().add_attribute("action", "try_register_currency"))
}

pub fn try_update_currency_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom_or_address: String,
    owner: Option<String>,
    whitelist: Option<Vec<String>>,
    permissionless_burning: Option<bool>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let mut is_currency_info_updated = false;
    let mut currency_info = CURRENCIES
        .load(deps.storage, &denom_or_address)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::simple(&currency_info.owner).assert(&sender)?;

    if let Some(x) = owner {
        let block_time = env.block.time.seconds();
        let new_owner = &deps.api.addr_validate(&x)?;
        let mut transfer_owner_state_list =
            TRANSFER_OWNER_STATE.load(deps.storage).unwrap_or_default();
        transfer_owner_state_list
            .retain(|(current_denom_or_address, _)| current_denom_or_address != &denom_or_address);

        transfer_owner_state_list.push((
            denom_or_address.clone(),
            TransferAdminState::new(new_owner, block_time + TRANSFER_ADMIN_TIMEOUT),
        ));

        TRANSFER_OWNER_STATE.save(deps.storage, &transfer_owner_state_list)?;

        is_currency_info_updated = true;
    }

    if let Some(x) = whitelist {
        currency_info.whitelist = x
            .iter()
            .map(|x| deps.api.addr_validate(x))
            .collect::<StdResult<Vec<Addr>>>()?;
        is_currency_info_updated = true;
    }

    if let Some(x) = permissionless_burning {
        currency_info.permissionless_burning = x;
        is_currency_info_updated = true;
    }

    // don't allow empty messages
    if !is_currency_info_updated {
        Err(ContractError::NoParameters)?;
    }

    CURRENCIES.save(deps.storage, &denom_or_address, &currency_info)?;

    Ok(Response::new().add_attribute("action", "try_update_currency_info"))
}

pub fn try_update_metadata_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let currency_info = CURRENCIES
        .load(deps.storage, &denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::simple(&currency_info.owner).assert(&sender)?;

    let sender = env.contract.address.to_string();
    let Metadata {
        description,
        denom_units,
        base,
        display,
        name,
        symbol,
        uri,
        uri_hash,
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
            uri: uri.unwrap_or_default(),
            uri_hash: uri_hash.unwrap_or_default(),
        }),
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_update_metadata_native"))
}

pub fn try_update_metadata_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    project: Option<String>,
    description: Option<String>,
    logo: Option<Logo>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "try_update_metadata_cw20");
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let currency_info = CURRENCIES
        .load(deps.storage, &address)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::simple(&currency_info.owner).assert(&sender)?;

    if project.is_some() || description.is_some() {
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address.to_owned(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing: None,
            })?,
            funds: vec![],
        }));
    }

    if let Some(x) = logo {
        response = response.add_message(get_upload_logo_msg(address, x)?);
    }

    // don't allow empty messages
    if response.messages.is_empty() {
        Err(ContractError::NoParameters)?;
    }

    Ok(response)
}

pub fn try_exclude_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let currency_info = CURRENCIES
        .load(deps.storage, &denom)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::simple(&currency_info.owner).assert(&sender)?;

    let token_count = TOKEN_COUNT.load(deps.storage, &currency_info.owner)? - 1;
    if token_count == 0 {
        TOKEN_COUNT.remove(deps.storage, &currency_info.owner);
    } else {
        TOKEN_COUNT.save(deps.storage, &currency_info.owner, &token_count)?;
    }

    CURRENCIES.remove(deps.storage, &denom);

    let current_admin = env.contract.address.to_string();
    let new_admin = currency_info.owner.to_string();
    let msg: CosmosMsg = OsmosisFactory::MsgChangeAdmin {
        denom,
        sender: current_admin,
        new_admin,
    }
    .into();

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_exclude_native"))
}

pub fn try_exclude_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let currency_info = CURRENCIES
        .load(deps.storage, &address)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::simple(&currency_info.owner).assert(&sender)?;

    let token_count = TOKEN_COUNT.load(deps.storage, &currency_info.owner)? - 1;
    if token_count == 0 {
        TOKEN_COUNT.remove(deps.storage, &currency_info.owner);
    } else {
        TOKEN_COUNT.save(deps.storage, &currency_info.owner, &token_count)?;
    }

    CURRENCIES.remove(deps.storage, &address);

    let new_admin = currency_info.owner.to_string();
    let msg_list = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address.clone(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(new_admin.clone()),
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address.clone(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::UpdateMarketing {
                project: None,
                description: None,
                marketing: Some(new_admin.clone()),
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
            contract_addr: address,
            admin: new_admin,
        }),
    ];

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attribute("action", "try_exclude_cw20"))
}

pub fn try_update_faucet_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom_or_address: String,
    claimable_amount: Option<Uint128>,
    claim_cooldown: Option<u64>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let currency_info = CURRENCIES
        .load(deps.storage, &denom_or_address)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::simple(&currency_info.owner).assert(&sender)?;

    let mut is_config_updated = false;
    let mut config = FAUCET_CONFIG
        .load(deps.storage, &denom_or_address)
        .unwrap_or_default();

    if let Some(x) = claimable_amount {
        config.claimable_amount = x;
        is_config_updated = true;
    }

    if let Some(x) = claim_cooldown {
        config.claim_cooldown = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    FAUCET_CONFIG.save(deps.storage, &denom_or_address, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_faucet_config"))
}

pub fn try_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom_or_address: String,
    amount: Uint128,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "try_mint");
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let currency_info = CURRENCIES
        .load(deps.storage, &denom_or_address)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::specified(deps.api, &currency_info.whitelist)?.assert(&sender)?;

    let creator = &env.contract.address;
    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender)
        .to_string();

    if currency_info.currency.token.is_native() {
        let amount = coin(amount.u128(), denom_or_address);

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
            contract_addr: denom_or_address,
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint { recipient, amount })?,
            funds: vec![],
        });

        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_mint_multiple(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom_or_address: String,
    account_and_amount_list: Vec<(String, Uint128)>,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "try_mint_multiple");
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let creator = &env.contract.address;
    let currency_info = CURRENCIES
        .load(deps.storage, &denom_or_address)
        .map_err(|_| ContractError::AssetIsNotFound)?;
    Auth::specified(deps.api, &currency_info.whitelist)?.assert(&sender)?;

    let total_amount = account_and_amount_list
        .iter()
        .fold(Uint128::zero(), |acc, (_, cur)| acc + cur);

    if currency_info.currency.token.is_native() {
        let msg: CosmosMsg = OsmosisFactory::MsgMint {
            sender: creator.to_string(),
            amount: Some(coin(total_amount.u128(), denom_or_address.clone()).into()),
            mint_to_address: creator.to_string(),
        }
        .into();
        response = response.add_message(msg);
    }

    for (recipient, amount) in account_and_amount_list {
        let msg = if currency_info.currency.token.is_native() {
            CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: coins(amount.u128(), denom_or_address.clone()),
            })
        } else {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: denom_or_address.clone(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint { recipient, amount })?,
                funds: vec![],
            })
        };
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
    check_pause_state(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "try_burn");
    let InfoResp {
        sender,
        asset_amount,
        asset_token,
    } = Funds::single(sender, amount).check(deps.api, &info)?;
    let creator = env.contract.address.to_string();
    let denom_or_address = asset_token.get_symbol();
    let currency_info = CURRENCIES
        .load(deps.storage, &denom_or_address)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    if !currency_info.permissionless_burning && !currency_info.whitelist.contains(&sender) {
        Err(ContractError::Auth(AuthError::Unauthorized))?;
    }

    if asset_token.is_native() {
        let amount = coin(asset_amount.u128(), denom_or_address);
        let msg: CosmosMsg = OsmosisFactory::MsgBurn {
            sender: creator.clone(),
            amount: Some(amount.into()),
            burn_from_address: creator,
        }
        .into();

        response = response.add_message(msg);
    } else {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: denom_or_address,
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Burn {
                amount: asset_amount,
            })?,
            funds: vec![],
        });

        response = response.add_message(msg);
    }

    Ok(response)
}

pub fn try_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom_or_address: String,
) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "try_claim");
    let InfoResp { sender, .. } = Funds::empty().check(deps.api, &info)?;
    let creator = &env.contract.address;
    let block_time = env.block.time.seconds();
    let currency_info = CURRENCIES
        .load(deps.storage, &denom_or_address)
        .map_err(|_| ContractError::AssetIsNotFound)?;

    let FaucetConfig {
        claimable_amount,
        claim_cooldown,
    } = FAUCET_CONFIG
        .load(deps.storage, &denom_or_address)
        .unwrap_or_default();
    let last_claimed = LAST_CLAIM_DATE
        .load(deps.storage, (&sender, &denom_or_address))
        .unwrap_or_default();

    if claimable_amount.is_zero() {
        Err(ContractError::FaucetIsDisabled)?;
    }

    if block_time < last_claimed + claim_cooldown {
        Err(ContractError::ClaimCooldown {
            remaining_time_in_mins: (last_claimed + claim_cooldown - block_time) / 60,
        })?;
    }

    LAST_CLAIM_DATE.save(deps.storage, (&sender, &denom_or_address), &block_time)?;

    if currency_info.currency.token.is_native() {
        let amount = coin(claimable_amount.u128(), denom_or_address);

        let msg_list = vec![
            OsmosisFactory::MsgMint {
                sender: creator.to_string(),
                amount: Some(amount.clone().into()),
                mint_to_address: creator.to_string(),
            }
            .into(),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: sender.to_string(),
                amount: vec![amount],
            }),
        ];

        response = response.add_messages(msg_list);
    } else {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: denom_or_address,
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint {
                recipient: sender.to_string(),
                amount: claimable_amount,
            })?,
            funds: vec![],
        });

        response = response.add_message(msg);
    }

    Ok(response)
}

fn get_full_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{creator}/{subdenom}")
}

/// user actions are disabled when the contract is paused
fn check_pause_state(storage: &dyn Storage) -> StdResult<()> {
    if IS_PAUSED.load(storage)? {
        Err(ContractError::ContractIsPaused)?;
    }

    Ok(())
}
