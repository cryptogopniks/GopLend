use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::{dec256_to_uint128, u128_to_dec, u128_to_dec256},
    error::ContractError,
    lending_platform::{
        state::{
            BALANCES, BGL_CURRENCY_DECIMALS, COLLECTIONS, CONFIG, CW20_NAME, CW20_SYMBOL,
            IS_LOCKED, PLATFORM_REVENUE, PROPOSALS, PROPOSAL_COUNTER, SAVE_CW20_ADDRESS_REPLY,
        },
        types::{Collection, Config, Proposal, ProposalStatus, ProposalType},
    },
    utils::{check_funds, get_transfer_msg, unwrap_field, Attrs, AuthType, FundsType},
};

pub fn try_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;

    let Config {
        main_currency,
        bgl_currency,
        ..
    } = CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(bgl_currency, "bgl_currency")?;

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update contract balances
    let mut balances = BALANCES.load(deps.storage)?;

    // initially 1 bgl_token == 1 main_token
    let decimals_multiplier = u128_to_dec256(10u128.pow(bgl_currency.decimals as u32))
        / u128_to_dec256(10u128.pow(main_currency.decimals as u32));

    let bgl_token_amount = dec256_to_uint128(if balances.total_bgl_currency_supply.is_zero() {
        decimals_multiplier * u128_to_dec256(asset_amount)
    } else {
        decimals_multiplier * u128_to_dec256(asset_amount * balances.total_bgl_currency_supply)
            / u128_to_dec256(balances.total_main_currency_pool)
    });

    balances.total_deposited += asset_amount;
    balances.total_main_currency_pool += asset_amount;
    balances.total_bgl_currency_supply += bgl_token_amount;

    BALANCES.save(deps.storage, &balances)?;

    // mint bgl_token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bgl_currency.token.try_get_cw20()?.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint {
            recipient: sender_address.to_string(),
            amount: bgl_token_amount,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_deposit")]))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    oracle: Option<String>,
    main_currency: Option<Currency<TokenUnverified>>,
    bgl_currency: Option<Currency<TokenUnverified>>,
    cw20_base_code_id: Option<u64>,
    unbonding_period: Option<u64>,
    borrow_apr: Option<Decimal>,
    borrow_fee_rate: Option<Decimal>,
    liquidation_fee_rate: Option<Decimal>,
    discount_min_rate: Option<Decimal>,
    discount_max_rate: Option<Decimal>,
    bid_min_rate: Option<Decimal>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    let mut attrs = Attrs::init("try_update_config");

    CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
        if let Some(x) = owner {
            config.owner = Some(deps.api.addr_validate(&x)?);
            attrs.push(("owner".to_string(), x));
        }

        if let Some(x) = oracle {
            config.oracle = Some(deps.api.addr_validate(&x)?);
            attrs.push(("oracle".to_string(), x));
        }

        if let Some(x) = main_currency {
            config.main_currency = Currency::new(&x.token.verify(deps.api)?, x.decimals);
            attrs.push(("main_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = bgl_currency {
            config.bgl_currency = Some(Currency::new(&x.token.verify(deps.api)?, x.decimals));
            attrs.push(("bgl_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = cw20_base_code_id {
            config.cw20_base_code_id = x;
            attrs.push(("cw20_base_code_id".to_string(), x.to_string()));
        }

        if let Some(x) = unbonding_period {
            config.unbonding_period = x;
            attrs.push(("unbonding_period".to_string(), x.to_string()));
        }

        if let Some(x) = borrow_apr {
            config.borrow_apr = x;
            attrs.push(("borrow_apr".to_string(), x.to_string()));
        }

        if let Some(x) = borrow_fee_rate {
            config.borrow_fee_rate = x;
            attrs.push(("borrow_fee_rate".to_string(), x.to_string()));
        }

        if let Some(x) = liquidation_fee_rate {
            config.liquidation_fee_rate = x;
            attrs.push(("liquidation_fee_rate".to_string(), x.to_string()));
        }

        if let Some(x) = discount_min_rate {
            config.discount_min_rate = x;
            attrs.push(("discount_min_rate".to_string(), x.to_string()));
        }

        if let Some(x) = discount_max_rate {
            config.discount_max_rate = x;
            attrs.push(("discount_max_rate".to_string(), x.to_string()));
        }

        if let Some(x) = bid_min_rate {
            config.bid_min_rate = x;
            attrs.push(("bid_min_rate".to_string(), x.to_string()));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
}

pub fn try_create_bgl_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    let creator = env.contract.address;
    let config = CONFIG.load(deps.storage)?;

    if config.bgl_currency.is_some() {
        Err(ContractError::AddressExists)?;
    }

    let cw20_msg = cw20_base::msg::InstantiateMsg {
        name: format!("{}", CW20_NAME),
        symbol: CW20_SYMBOL.to_string(),
        decimals: BGL_CURRENCY_DECIMALS,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: creator.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(creator.to_string()),
        code_id: config.cw20_base_code_id,
        label: format!("CryptoGopniks {}", CW20_SYMBOL),
        msg: to_json_binary(&cw20_msg)?,
        funds: vec![],
    });

    let submsg = SubMsg::reply_on_success(msg, SAVE_CW20_ADDRESS_REPLY);

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attributes([("action", "try_create_bgl_token")]))
}

pub fn try_deposit_reserve_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    let Config {
        main_currency,
        bgl_currency,
        ..
    } = CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(bgl_currency, "bgl_currency")?;

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update contract balances
    let mut balances = BALANCES.load(deps.storage)?;

    // initially 1 bgl_token == 1 main_token
    let decimals_multiplier = u128_to_dec256(10u128.pow(bgl_currency.decimals as u32))
        / u128_to_dec256(10u128.pow(main_currency.decimals as u32));

    let bgl_token_amount = dec256_to_uint128(if balances.total_bgl_currency_supply.is_zero() {
        decimals_multiplier * u128_to_dec256(asset_amount)
    } else {
        decimals_multiplier * u128_to_dec256(asset_amount * balances.total_bgl_currency_supply)
            / u128_to_dec256(balances.total_main_currency_pool)
    });

    // it doesn't affect on deposit APR
    balances.total_main_currency_pool += asset_amount;
    balances.reserve_main_currency_pool += asset_amount;
    balances.total_bgl_currency_supply += bgl_token_amount;
    balances.reserve_bgl_currency_supply += bgl_token_amount;

    BALANCES.save(deps.storage, &balances)?;

    // mint bgl_token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bgl_currency.token.try_get_cw20()?.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::Mint {
            recipient: sender_address.to_string(),
            amount: bgl_token_amount,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_deposit_reserve_liquidity")]))
}

pub fn try_lock(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    IS_LOCKED.update(deps.storage, |_| -> StdResult<bool> { Ok(true) })?;

    Ok(Response::new().add_attributes([("action", "try_lock")]))
}

pub fn try_unlock(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    IS_LOCKED.update(deps.storage, |_| -> StdResult<bool> { Ok(false) })?;

    Ok(Response::new().add_attributes([("action", "try_unlock")]))
}

pub fn try_distribute_funds(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address_and_weight_list: Vec<(String, Decimal)>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    // verify weights
    // check if all weights are in range [0, 1]
    if address_and_weight_list
        .iter()
        .any(|(_, weight)| weight > &Decimal::one())
    {
        Err(ContractError::WeightIsOutOfRange)?;
    }

    // check if sum of weights is equal one
    let weights_sum = address_and_weight_list
        .iter()
        .fold(Decimal::zero(), |acc, (_, weight)| acc + weight);

    if weights_sum != Decimal::one() {
        Err(ContractError::WeightsAreUnbalanced)?;
    }

    // create send messages and update state
    let mut funds_list = PLATFORM_REVENUE.load(deps.storage)?;
    let mut msg_list: Vec<CosmosMsg> = vec![];

    let address_and_weight_list = address_and_weight_list
        .into_iter()
        .map(|(address, weight)| -> StdResult<(Addr, Decimal)> {
            let recipient = deps.api.addr_validate(&address)?;
            Ok((recipient, weight))
        })
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    for funds_list_item in funds_list.iter_mut() {
        let mut amount_to_send = Uint128::zero();

        for (recipient, weight) in &address_and_weight_list {
            let amount = (u128_to_dec(funds_list_item.amount.to_owned()) * weight).to_uint_floor();
            amount_to_send += amount;

            let msg = get_transfer_msg(recipient, amount, &funds_list_item.currency.token)?;
            msg_list.push(msg);
        }

        funds_list_item.amount -= amount_to_send;
    }

    PLATFORM_REVENUE.save(deps.storage, &funds_list)?;

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes([("action", "try_distribute_funds")]))
}

pub fn try_remove_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    // update state
    let collection_address = &deps.api.addr_validate(&address)?;
    COLLECTIONS.remove(deps.storage, collection_address);

    Ok(Response::new().add_attributes([("action", "try_remove_collection")]))
}

pub fn try_create_proposal(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposal: Proposal<String, TokenUnverified>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    // verify proposal fields
    let Proposal {
        proposal_type,
        listing_price,
        ..
    } = proposal;

    let proposal_type: ProposalType<Addr> = match proposal_type {
        ProposalType::AddCollection {
            collection_address,
            collection: Collection { name, owner },
        } => {
            let collection_address = deps.api.addr_validate(&collection_address)?;
            let owner = deps.api.addr_validate(&owner)?;

            let is_collection_found = COLLECTIONS
                .range(deps.storage, None, None, Order::Ascending)
                .flatten()
                .any(|(current_address, current_collection)| {
                    (current_address == collection_address) || (current_collection.name == name)
                });

            if is_collection_found {
                Err(ContractError::CollectionDuplication)?;
            }

            ProposalType::AddCollection {
                collection_address,
                collection: Collection { name, owner },
            }
        }
        ProposalType::UpdateCollection {
            collection_address,
            new_collection_address,
            new_collection: Collection { name, owner },
        } => {
            let collection_address = deps.api.addr_validate(&collection_address)?;
            let new_collection_address = match new_collection_address {
                Some(x) => Some(deps.api.addr_validate(&x)?),
                _ => None,
            };
            let owner = deps.api.addr_validate(&owner)?;

            let is_collection_found = COLLECTIONS
                .range(deps.storage, None, None, Order::Ascending)
                .flatten()
                .any(|(current_address, _current_collection)| {
                    new_collection_address.is_some()
                        && (current_address == new_collection_address.clone().unwrap())
                });

            if is_collection_found {
                Err(ContractError::CollectionDuplication)?;
            }

            ProposalType::UpdateCollection {
                collection_address,
                new_collection_address,
                new_collection: Collection { name, owner },
            }
        }
    };

    let token = listing_price.currency.token.verify(deps.api)?;
    let proposal: Proposal<Addr, Token> = Proposal {
        proposal_status: Some(ProposalStatus::Active),
        proposal_type,
        listing_price: Funds::new(
            listing_price.amount,
            &Currency::new(&token, listing_price.currency.decimals),
        ),
    };

    // update list of proposals
    let proposal_counter = PROPOSAL_COUNTER.load(deps.storage)?;
    PROPOSALS.save(deps.storage, proposal_counter, &proposal)?;
    PROPOSAL_COUNTER.save(deps.storage, &(proposal_counter + 1))?;

    Ok(Response::new().add_attributes([
        ("action", "try_create_proposal"),
        ("proposal_id", &proposal_counter.to_string()),
    ]))
}

pub fn try_reject_proposal(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    // update proposal status
    PROPOSALS.update(
        deps.storage,
        id,
        |proposal| -> StdResult<Proposal<Addr, Token>> {
            let proposal = unwrap_field(proposal, "proposal")?;

            // verify proposal status
            verify_proposal_status(&proposal, ProposalStatus::Active)?;

            Ok(Proposal {
                proposal_status: Some(ProposalStatus::Rejected),
                ..proposal
            })
        },
    )?;

    Ok(Response::new().add_attributes([("action", "try_reject_proposal")]))
}

pub fn try_accept_proposal(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;

    let proposal = PROPOSALS.load(deps.storage, id)?;

    // verify funds
    if (asset_amount != proposal.listing_price.amount)
        || (asset_info != proposal.listing_price.currency.token)
    {
        Err(ContractError::WrongFundsCombination)?;
    }

    // verify proposal status
    verify_proposal_status(&proposal, ProposalStatus::Active)?;

    let (collection_address, new_collection_address, collection) = match &proposal.proposal_type {
        ProposalType::AddCollection {
            collection_address,
            collection,
        } => {
            let is_collection_found = COLLECTIONS
                .range(deps.storage, None, None, Order::Ascending)
                .flatten()
                .any(|(current_address, current_collection)| {
                    (current_address == collection_address)
                        || (current_collection.name == collection.name)
                });

            if is_collection_found {
                Err(ContractError::CollectionDuplication)?;
            }

            (collection_address, None, collection)
        }
        ProposalType::UpdateCollection {
            collection_address,
            new_collection_address,
            new_collection,
        } => {
            let collections = &COLLECTIONS
                .range(deps.storage, None, None, Order::Ascending)
                .flatten()
                .collect::<Vec<(Addr, Collection<Addr>)>>();

            let is_collection_found = collections
                .iter()
                .any(|(current_address, _)| (current_address == collection_address));

            if !is_collection_found {
                Err(ContractError::CollectionIsNotFound)?;
            }

            let is_collection_found =
                collections
                    .iter()
                    .any(|(current_address, _current_collection)| {
                        new_collection_address.is_some()
                            && (current_address == new_collection_address.clone().unwrap())
                    });

            if is_collection_found {
                Err(ContractError::CollectionDuplication)?;
            }

            (
                collection_address,
                new_collection_address.as_ref(),
                new_collection,
            )
        }
    };

    // verify sender
    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(collection.owner.clone())],
        },
    )?;

    // add/update collection
    match new_collection_address {
        Some(x) => {
            COLLECTIONS.remove(deps.storage, collection_address);
            COLLECTIONS.save(deps.storage, x, collection)?;
        }
        _ => COLLECTIONS.save(deps.storage, collection_address, collection)?,
    };

    // update funds
    let mut funds_list = PLATFORM_REVENUE.load(deps.storage)?;

    let funds = funds_list
        .iter()
        .find(|Funds { currency, .. }| currency.token == asset_info);

    if funds.is_none() {
        funds_list.push(Funds::new(
            Uint128::zero(),
            &proposal.listing_price.currency,
        ));
    }

    for funds_list_item in funds_list.iter_mut() {
        if funds_list_item.currency.token == asset_info {
            funds_list_item.amount += asset_amount;
        }
    }

    PLATFORM_REVENUE.save(deps.storage, &funds_list)?;

    // update proposal status
    PROPOSALS.save(
        deps.storage,
        id,
        &Proposal {
            proposal_status: Some(ProposalStatus::Accepted),
            ..proposal
        },
    )?;

    Ok(Response::new().add_attributes([("action", "try_accept_proposal")]))
}

fn check_lockout(deps: Deps) -> StdResult<()> {
    if IS_LOCKED.load(deps.storage)? {
        Err(ContractError::ContractIsLocked)?;
    }

    Ok(())
}

fn verify_proposal_status(
    proposal: &Proposal<Addr, Token>,
    expected: ProposalStatus,
) -> StdResult<()> {
    let proposal_status = unwrap_field(proposal.proposal_status.as_ref(), "proposal_status")?;

    if proposal_status != &expected {
        Err(ContractError::WrongProposalStatus)?;
    }

    Ok(())
}

fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let Config { admin, owner, .. } = CONFIG.load(deps.storage)?;
    let owner = unwrap_field(owner, "owner");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrOwner => {
            if !((sender == admin) || (owner.is_ok() && sender == owner?)) {
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
        AuthType::AdminOrOwnerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (owner.is_ok() && sender == owner?) || is_included) {
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
