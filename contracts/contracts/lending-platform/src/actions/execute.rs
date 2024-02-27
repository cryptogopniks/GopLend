use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Decimal, DepsMut, Empty, Env, MessageInfo, Order, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};

use cw721::Cw721ExecuteMsg;

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::{
        state::{
            BALANCES, BGL_CURRENCY_DECIMALS, BORROWERS, COLLATERALS, COLLECTIONS, CONFIG,
            CW20_NAME, CW20_SYMBOL, IS_LOCKED, LIQUIDATION_BIDS, LIQUIDATORS, PAGINATION_CONFIG,
            PLATFORM_REVENUE, PROPOSALS, PROPOSAL_COUNTER, SAVE_CW20_ADDRESS_REPLY, UNBONDERS,
        },
        types::{
            Balances, Collateral, Collection, CollectionInfo, Config, LiquidationBid, Liquidator,
            PaginationConfig, Proposal, ProposalStatus, ProposalType, Unbonder, Vault,
        },
    },
    utils::{check_funds, get_transfer_msg, unwrap_field, Attrs, AuthType, FundsType},
};

use crate::{
    actions::query,
    helpers::{
        accumulate_expected_rewards, check_authorization, check_collections_holder, check_lockout,
        verify_proposal_status,
    },
    math::{
        calc_accumulated_loan, calc_bid_max_multiplier, calc_expected_rewards,
        calc_minted_bgl_token_amount, calc_unbonded_main_token_amount,
    },
};

pub fn try_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
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
    accumulate_expected_rewards(deps.storage, block_time)?;
    let mut balances = BALANCES.load(deps.storage)?;

    let bgl_token_amount = calc_minted_bgl_token_amount(
        asset_amount,
        balances.total_main_currency_pool,
        balances.total_bgl_currency_supply,
        balances.expected_accumulated_rewards,
        main_currency.decimals,
        bgl_currency.decimals,
    );

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

pub fn try_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let Config {
        main_currency,
        bgl_currency,
        ..
    } = CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(bgl_currency, "bgl_currency")?;

    if asset_info != bgl_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update balances
    accumulate_expected_rewards(deps.storage, block_time)?;
    let mut balances = BALANCES.load(deps.storage)?;

    let main_token_amount = calc_unbonded_main_token_amount(
        asset_amount,
        balances.total_main_currency_pool,
        balances.total_bgl_currency_supply,
        balances.expected_accumulated_rewards,
        main_currency.decimals,
        bgl_currency.decimals,
    );

    balances.total_unbonded += main_token_amount;

    BALANCES.save(deps.storage, &balances)?;

    // create unbonding position to withdraw main token later
    UNBONDERS.update(deps.storage, &sender_address, |x| -> StdResult<Unbonder> {
        let returned_and_unbonded = (
            asset_amount,
            Vault {
                amount: main_token_amount,
                creation_date: block_time,
            },
        );

        let unbonder = match x {
            Some(mut unbonder) => {
                // don't allow multiple vaults with same creation date
                let creation_date_list: Vec<u64> = unbonder
                    .returned_and_unbonded
                    .iter()
                    .map(|(_returned, unbonded)| unbonded.creation_date)
                    .collect();

                if creation_date_list.contains(&block_time) {
                    Err(ContractError::SameCreationDate)?;
                }

                unbonder.returned_and_unbonded.push(returned_and_unbonded);

                unbonder
            }
            None => Unbonder {
                returned_and_unbonded: vec![returned_and_unbonded],
            },
        };

        Ok(unbonder)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "try_unbond")]))
}

pub fn try_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let Config {
        main_currency,
        bgl_currency,
        unbonding_period,
        ..
    } = CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(bgl_currency, "bgl_currency")?;

    let mut amount_to_burn = Uint128::zero();
    let mut amount_to_withdraw = Uint128::zero();
    let mut depositor = UNBONDERS.load(deps.storage, &sender_address)?;

    depositor.returned_and_unbonded = depositor
        .returned_and_unbonded
        .into_iter()
        .filter(|(returned, unbonded)| {
            if block_time >= unbonded.creation_date + unbonding_period {
                amount_to_burn += returned;
                amount_to_withdraw += unbonded.amount;

                false
            } else {
                true
            }
        })
        .collect();

    UNBONDERS.save(deps.storage, &sender_address, &depositor)?;

    if amount_to_withdraw.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    // update balances, bgl to main ratio was fixed on Unbond
    let mut balances = BALANCES.load(deps.storage)?;

    balances.total_unbonded -= amount_to_withdraw;
    balances.total_main_currency_pool -= amount_to_withdraw;
    balances.total_bgl_currency_supply -= amount_to_burn;

    BALANCES.save(deps.storage, &balances)?;

    // burn bgl token and send main token
    let msg_list = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bgl_currency.token.try_get_cw20()?.to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Burn {
                amount: amount_to_burn,
            })?,
            funds: vec![],
        }),
        get_transfer_msg(&sender_address, amount_to_withdraw, &main_currency.token)?,
    ];

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "try_withdraw")]))
}

pub fn try_deposit_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collections: Vec<CollectionInfo<String>>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    // check if nfts are on user balance
    check_collections_holder(deps.as_ref(), &sender_address, &collections)?;

    let mut msg_list: Vec<CosmosMsg> = vec![];

    for CollectionInfo {
        collection_address,
        mut token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }

        // remove token duplication
        token_id_list.sort_unstable();
        token_id_list.dedup();

        // add transfer msgs
        for token_id in &token_id_list {
            let cw721_msg = Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_id.to_string(),
            };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection_address.to_string(),
                msg: to_json_binary(&cw721_msg)?,
                funds: vec![],
            });

            msg_list.push(msg);
        }

        // TODO: check proper removal on other Map storages
        // update collaterals
        COLLATERALS.update(
            deps.storage,
            &collection_address,
            |x| -> StdResult<Vec<Collateral>> {
                let collaterals = match x {
                    Some(mut collaterals) => {
                        let current_collateral = collaterals
                            .iter()
                            .find(|collateral| collateral.owner == sender_address);

                        if current_collateral.is_none() {
                            collaterals.push(Collateral {
                                owner: sender_address.to_owned(),
                                token_id_list: vec![],
                            });
                        }

                        collaterals = collaterals
                            .into_iter()
                            .map(|mut collateral| {
                                collateral.token_id_list =
                                    vec![collateral.token_id_list, token_id_list.clone()].concat();
                                collateral.token_id_list.sort_unstable();
                                collateral.token_id_list.dedup();

                                collateral
                            })
                            .collect();

                        collaterals
                    }
                    None => vec![Collateral {
                        owner: sender_address.clone(),
                        token_id_list,
                    }],
                };

                Ok(collaterals)
            },
        )?;
    }

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "try_deposit_collateral")]))
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collections: Vec<CollectionInfo<String>>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    // check if nfts are on contract balance
    check_collections_holder(deps.as_ref(), &env.contract.address, &collections)?;

    let mut msg_list: Vec<CosmosMsg> = vec![];

    for CollectionInfo {
        collection_address,
        mut token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }

        // remove token duplication
        token_id_list.sort_unstable();
        token_id_list.dedup();

        // add transfer msgs
        for token_id in &token_id_list {
            let cw721_msg = Cw721ExecuteMsg::TransferNft {
                recipient: sender_address.to_string(),
                token_id: token_id.to_string(),
            };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection_address.to_string(),
                msg: to_json_binary(&cw721_msg)?,
                funds: vec![],
            });

            msg_list.push(msg);
        }

        // update collaterals
        let collaterals: Vec<Collateral> = COLLATERALS
            .load(deps.storage, &collection_address)?
            .into_iter()
            .map(|mut collateral| {
                if collateral.owner == sender_address {
                    collateral.token_id_list = collateral
                        .token_id_list
                        .into_iter()
                        .filter(|id| !token_id_list.contains(id))
                        .collect();

                    collateral
                } else {
                    collateral
                }
            })
            .filter(|collateral| !collateral.token_id_list.is_empty())
            .collect();

        if collaterals.is_empty() {
            COLLATERALS.remove(deps.storage, &collection_address);
        } else {
            COLLATERALS.save(deps.storage, &collection_address, &collaterals)?;
        }
    }

    // check ltv
    let ltv = query::query_ltv(deps.as_ref(), env, sender_address.to_string())?;
    let Config {
        discount_max_rate,
        discount_min_rate,
        bid_min_rate,
        ..
    } = CONFIG.load(deps.storage)?;

    let ltv_max = calc_bid_max_multiplier(bid_min_rate, discount_max_rate, discount_min_rate);

    if ltv > ltv_max {
        Err(ContractError::ExceededLtv)?;
    }

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "try_withdraw_collateral")]))
}

// TODO: reduce borrowers amount by adding limitations (collateral size)
// TODO: limit total borrowed amount to avoid zero available to withdraw amount
pub fn try_borrow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let Config {
        main_currency,
        borrow_apr,
        discount_max_rate,
        discount_min_rate,
        bid_min_rate,
        ..
    } = CONFIG.load(deps.storage)?;
    let mut borrower = BORROWERS.load(deps.storage, &sender_address)?;

    borrower.accumulated_loan = calc_accumulated_loan(
        borrow_apr,
        borrower.accumulated_loan,
        borrower.loan.amount,
        block_time,
        borrower.loan.creation_date,
    );
    borrower.loan = Vault {
        creation_date: block_time,
        amount,
    };

    // check ltv
    let ltv = query::query_ltv(deps.as_ref(), env, sender_address.to_string())?;
    let ltv_max = calc_bid_max_multiplier(bid_min_rate, discount_max_rate, discount_min_rate);

    if ltv > ltv_max {
        Err(ContractError::ExceededLtv)?;
    }

    // update states
    BORROWERS.save(deps.storage, &sender_address, &borrower)?;

    accumulate_expected_rewards(deps.storage, block_time)?;
    BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
        x.total_borrowed.amount += amount;

        Ok(x)
    })?;

    // create send msg
    let msg = get_transfer_msg(&sender_address, amount, &main_currency.token)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_borrow")]))
}

pub fn try_repay(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let Config {
        main_currency,
        borrow_apr,
        ..
    } = CONFIG.load(deps.storage)?;

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    let mut borrower = BORROWERS.load(deps.storage, &sender_address)?;

    borrower.accumulated_loan = calc_accumulated_loan(
        borrow_apr,
        borrower.accumulated_loan,
        borrower.loan.amount,
        block_time,
        borrower.loan.creation_date,
    ) - asset_amount;

    borrower.loan = Vault {
        creation_date: block_time,
        amount: Uint128::zero(),
    };

    // update states
    BORROWERS.save(deps.storage, &sender_address, &borrower)?;

    accumulate_expected_rewards(deps.storage, block_time)?;
    BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
        x.total_borrowed.amount -= asset_amount;

        Ok(x)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "try_repay")]))
}

// TODO: limit max amount of bids per liquidator at 5 collections with 5 tokens each
pub fn try_place_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collections: Vec<CollectionInfo<String>>,
    discount: Decimal,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let Config { main_currency, .. } = CONFIG.load(deps.storage)?;

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    let mut collection_addresses: Vec<Addr> = vec![];

    for CollectionInfo {
        collection_address,
        mut token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }
        collection_addresses.push(collection_address.clone());

        // remove token duplication
        token_id_list.sort_unstable();
        token_id_list.dedup();

        // update bids
        LIQUIDATION_BIDS.update(
            deps.storage,
            &collection_address,
            |x| -> StdResult<Vec<LiquidationBid>> {
                let bid = LiquidationBid {
                    creation_date: block_time,
                    liquidator: sender_address.clone(),
                    token_id_list: token_id_list.clone(),
                    amount: asset_amount,
                    discount,
                };

                let bids = match x {
                    Some(mut y) => {
                        let bids_by_liquidator: Vec<LiquidationBid> = y
                            .iter()
                            .cloned()
                            .filter(|z| z.liquidator == sender_address)
                            .collect();

                        // check if there is no token duplication
                        if bids_by_liquidator.iter().any(|bid| {
                            bid.token_id_list
                                .iter()
                                .any(|token| token_id_list.contains(token))
                        }) {
                            Err(ContractError::NftDuplication)?;
                        }

                        // TODO: check if it's required
                        // don't allow same date bids
                        if bids_by_liquidator
                            .iter()
                            .any(|bid| bid.creation_date == block_time)
                        {
                            Err(ContractError::SameCreationDate)?;
                        }

                        y.push(bid);

                        y
                    }
                    None => vec![bid],
                };

                Ok(bids)
            },
        )?;
    }

    LIQUIDATORS.update(
        deps.storage,
        &sender_address,
        |x| -> StdResult<Liquidator> {
            let liquidator = match x {
                Some(mut y) => {
                    y.collection_addresses =
                        vec![y.collection_addresses, collection_addresses].concat();
                    y.collection_addresses.sort_unstable();
                    y.collection_addresses.dedup();

                    y
                }
                None => Liquidator {
                    collection_addresses,
                },
            };

            Ok(liquidator)
        },
    )?;

    BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
        x.total_liquidation_bids += asset_amount;

        Ok(x)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "try_place_bid")]))
}

pub fn try_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collections: Vec<CollectionInfo<String>>,
    creation_date: u64,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let Config { main_currency, .. } = CONFIG.load(deps.storage)?;

    let mut amount_to_withdraw = Uint128::zero();
    let mut collection_addresses: Vec<Addr> = vec![];

    for CollectionInfo {
        collection_address,
        mut token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }
        collection_addresses.push(collection_address.clone());

        // remove token duplication
        token_id_list.sort_unstable();
        token_id_list.dedup();

        // update bids
        let liquidation_bids = LIQUIDATION_BIDS
            .load(deps.storage, &collection_address)
            .map_err(|_| ContractError::CollectionIsNotFound)?;

        let bids_by_liquidator: Vec<LiquidationBid> = liquidation_bids
            .iter()
            .cloned()
            .filter(|z| z.liquidator == sender_address)
            .collect();

        // check if all token_id_list are included in bid.token_id_list
        if !token_id_list.iter().all(|id| {
            bids_by_liquidator
                .iter()
                .any(|bid| bid.token_id_list.contains(id))
        }) {
            Err(ContractError::NftIsNotFound)?;
        }

        // remove bids
        let liquidation_bids = liquidation_bids
            .into_iter()
            .filter(|z| {
                if z.liquidator == sender_address && z.creation_date == creation_date {
                    amount_to_withdraw += z.amount;

                    false
                } else {
                    true
                }
            })
            .collect::<Vec<LiquidationBid>>();

        if !liquidation_bids.is_empty() {
            LIQUIDATION_BIDS.save(deps.storage, &collection_address, &liquidation_bids)?;
        } else {
            LIQUIDATION_BIDS.remove(deps.storage, &collection_address);
        }
    }

    BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
        x.total_liquidation_bids -= amount_to_withdraw;

        Ok(x)
    })?;

    let msg = get_transfer_msg(&sender_address, amount_to_withdraw, &main_currency.token)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_remove_bid")]))
}

pub fn try_update_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collections: Vec<CollectionInfo<String>>,
    creation_date: u64,
    bid_amount: Uint128,
    discount: Decimal,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;

    let (sender_address, asset_amount, asset_info) = if sender.is_some() && amount.is_some() {
        (
            deps.api.addr_validate(&sender.unwrap_or_default())?,
            amount.unwrap_or_default(),
            Token::new_cw20(&info.sender),
        )
    } else if sender.is_none() && amount.is_none() {
        (
            info.sender.clone(),
            Uint128::default(),
            Token::new_native(&String::default()),
        )
    } else {
        Err(ContractError::WrongFundsCombination)?
    };

    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let Config { main_currency, .. } = CONFIG.load(deps.storage)?;

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    let mut current_bid_value = Uint128::zero();
    let mut new_bid_value = Uint128::zero();
    let mut collection_addresses: Vec<Addr> = vec![];

    for CollectionInfo {
        collection_address,
        mut token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }
        collection_addresses.push(collection_address.clone());

        // remove token duplication
        token_id_list.sort_unstable();
        token_id_list.dedup();

        // update new bid value
        new_bid_value += bid_amount * Uint128::from(token_id_list.len() as u128);

        // update bids
        LIQUIDATION_BIDS.update(
            deps.storage,
            &collection_address,
            |x| -> StdResult<Vec<LiquidationBid>> {
                let bids = match x {
                    Some(mut y) => {
                        let bids_by_liquidator: Vec<LiquidationBid> = y
                            .iter()
                            .cloned()
                            .filter(|z| z.liquidator == sender_address)
                            .collect();

                        // check if all token_id_list are included in bid.token_id_list
                        if !token_id_list.iter().all(|id| {
                            bids_by_liquidator
                                .iter()
                                .any(|bid| bid.token_id_list.contains(id))
                        }) {
                            Err(ContractError::NftIsNotFound)?;
                        }

                        // replace bids
                        y = y
                            .into_iter()
                            .map(|z| {
                                if z.liquidator == sender_address
                                    && z.creation_date == creation_date
                                {
                                    // update current bid value
                                    current_bid_value += z.amount;

                                    LiquidationBid {
                                        creation_date: block_time, // decrease changed bid priority
                                        liquidator: sender_address.clone(),
                                        token_id_list: token_id_list.clone(),
                                        amount: bid_amount,
                                        discount,
                                    }
                                } else {
                                    z
                                }
                            })
                            .collect();

                        y
                    }
                    None => Err(ContractError::CollectionIsNotFound)?,
                };

                Ok(bids)
            },
        )?;
    }

    let response: Response<Empty> =
        Response::new().add_attributes(vec![("action", "try_update_bid")]);

    if new_bid_value >= current_bid_value && current_bid_value + asset_amount >= new_bid_value {
        let amount_to_return = current_bid_value + asset_amount - new_bid_value;

        BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
            x.total_liquidation_bids += new_bid_value - current_bid_value;

            Ok(x)
        })?;

        if !amount_to_return.is_zero() {
            let msg = get_transfer_msg(&sender_address, amount_to_return, &asset_info)?;

            return Ok(response.add_message(msg));
        }

        return Ok(response);
    }

    if new_bid_value < current_bid_value {
        let amount_to_return = current_bid_value - new_bid_value + asset_amount;

        BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
            x.total_liquidation_bids -= current_bid_value - new_bid_value;

            Ok(x)
        })?;

        if !amount_to_return.is_zero() {
            let msg = get_transfer_msg(&sender_address, amount_to_return, &asset_info)?;

            return Ok(response.add_message(msg));
        }

        return Ok(response);
    }

    Err(ContractError::WrongFundsCombination)
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    env: Env,
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
    pagination_amount: Option<u32>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    accumulate_expected_rewards(deps.storage, env.block.time.seconds())?;
    let balances = BALANCES.load(deps.storage)?;
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
            if !balances.total_main_currency_pool.is_zero() {
                Err(ContractError::ChangingCurrency)?;
            }

            config.main_currency = Currency::new(&x.token.verify(deps.api)?, x.decimals);
            attrs.push(("main_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = bgl_currency {
            if !balances.total_bgl_currency_supply.is_zero() {
                Err(ContractError::ChangingCurrency)?;
            }

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

    if let Some(x) = pagination_amount {
        let PaginationConfig {
            pagination_index, ..
        } = PAGINATION_CONFIG.load(deps.storage)?;
        PAGINATION_CONFIG.save(deps.storage, &PaginationConfig::new(x, &pagination_index))?;
        attrs.push(("pagination_amount".to_string(), x.to_string()));
    }

    Ok(Response::new().add_attributes(attrs))
}

// TODO: use separate minter contract
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
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrOwner)?;

    let block_time = env.block.time.seconds();
    let Config {
        main_currency,
        bgl_currency,
        borrow_apr,
        ..
    } = CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(bgl_currency, "bgl_currency")?;

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update contract balances
    let mut balances = BALANCES.load(deps.storage)?;

    let expected_rewards = calc_expected_rewards(
        block_time,
        balances.total_borrowed.creation_date,
        borrow_apr,
        balances.total_borrowed.amount,
    );
    let rewards = balances.expected_accumulated_rewards + expected_rewards;

    let bgl_token_amount = calc_minted_bgl_token_amount(
        asset_amount,
        balances.total_main_currency_pool,
        balances.total_bgl_currency_supply,
        rewards,
        main_currency.decimals,
        bgl_currency.decimals,
    );

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
