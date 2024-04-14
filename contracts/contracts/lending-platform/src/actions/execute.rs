use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Uint128, WasmMsg,
};

use cw721::Cw721ExecuteMsg;

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::{
        state::{
            ADDRESS_CONFIG, BALANCES, BORROWERS, COLLATERALS, COLLECTIONS, COMMON_CONFIG,
            IS_LOCKED, LIQUIDATION_BIDS, LIQUIDATION_BID_MAX_AMOUNT,
            LIQUIDATION_BID_MAX_TOKEN_AMOUNT, LIQUIDATORS, PLATFORM_REVENUE, PROPOSALS,
            PROPOSAL_COUNTER, RATE_CONFIG, UNBONDERS,
        },
        types::{
            AddressConfig, Balances, Borrower, Collateral, Collection, CollectionInfo,
            CommonConfig, LiquidationBid, LiquidationItem, Liquidator, Proposal, ProposalStatus,
            ProposalType, RateConfig, Unbonder, Vault,
        },
    },
    minter, oracle,
    utils::{
        add_funds_to_exec_msg, check_funds, get_transfer_msg, unwrap_field, Attrs, AuthType,
        FundsType,
    },
};

use crate::{
    actions::query,
    helpers::{
        accumulate_expected_rewards, check_authorization, check_collections_holder, check_lockout,
        increase_platform_revenue, update_balances, update_borrowers_storages,
        update_liquidation_storages, verify_proposal_status,
    },
    math,
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
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(common_config.bgl_currency, "bgl_currency")?;
    let minter_address = unwrap_field(address_config.minter, "minter")?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != common_config.main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update contract balances
    accumulate_expected_rewards(deps.storage, block_time)?;
    let mut balances = BALANCES.load(deps.storage)?;

    let bgl_token_amount = math::calc_minted_bgl_token_amount(
        asset_amount,
        balances.regular.main_pool,
        balances.regular.bgl_supply,
        balances.regular.accumulated_rewards,
        common_config.main_currency.decimals,
        bgl_currency.decimals,
    );

    balances.regular.deposited += asset_amount;
    balances.regular.main_pool += asset_amount;
    balances.regular.bgl_supply += bgl_token_amount;

    BALANCES.save(deps.storage, &balances)?;

    // mint bgl_token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: minter_address.to_string(),
        msg: to_json_binary(&minter::msg::ExecuteMsg::Mint {
            token: bgl_currency.token.into(),
            amount: bgl_token_amount,
            recipient: sender_address.to_string(),
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
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(common_config.bgl_currency, "bgl_currency")?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != bgl_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update balances
    accumulate_expected_rewards(deps.storage, block_time)?;
    let mut balances = BALANCES.load(deps.storage)?;

    let main_token_amount = math::calc_unbonded_main_token_amount(
        asset_amount,
        balances.regular.main_pool,
        balances.regular.bgl_supply,
        balances.regular.accumulated_rewards,
        common_config.main_currency.decimals,
        bgl_currency.decimals,
    );

    balances.regular.unbonded += main_token_amount;

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
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(common_config.bgl_currency, "bgl_currency")?;
    let minter_address = unwrap_field(address_config.minter, "minter")?;

    let mut amount_to_burn = Uint128::zero();
    let mut amount_to_withdraw = Uint128::zero();
    let mut unbonder = UNBONDERS.load(deps.storage, &sender_address)?;

    unbonder
        .returned_and_unbonded
        .retain(|(returned, unbonded)| {
            if block_time >= unbonded.creation_date + common_config.unbonding_period {
                amount_to_burn += returned;
                amount_to_withdraw += unbonded.amount;

                false
            } else {
                true
            }
        });

    if unbonder.returned_and_unbonded.is_empty() {
        UNBONDERS.remove(deps.storage, &sender_address);
    } else {
        UNBONDERS.save(deps.storage, &sender_address, &unbonder)?;
    }

    if amount_to_withdraw.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    // update balances, bgl to main ratio was fixed on Unbond
    let mut balances = BALANCES.load(deps.storage)?;

    // don't allow to withdraw if available liquidity is too low
    let withdrawers_reserve_fraction = math::calc_withdrawers_reserve_fraction(
        balances.reserve.deposited,
        common_config.borrowers_reserve_fraction_ratio,
    );

    let available_to_withdraw_reserve_liquidity =
        math::calc_available_to_withdraw_reserve_liquidity(
            withdrawers_reserve_fraction,
            balances.reserve.withdrawn,
        );

    if amount_to_withdraw > balances.regular.main_pool + available_to_withdraw_reserve_liquidity {
        Err(ContractError::LowAvailableLiquidity)?;
    }

    if amount_to_withdraw <= balances.regular.main_pool {
        balances.regular.main_pool -= amount_to_withdraw;
        balances.regular.deposited -= std::cmp::min(amount_to_withdraw, balances.regular.deposited);
    } else {
        balances.reserve.withdrawn += amount_to_withdraw - balances.regular.main_pool;
        balances.regular.main_pool = Uint128::zero();
        balances.regular.deposited = Uint128::zero();
    }

    balances.regular.unbonded -= amount_to_withdraw;
    balances.regular.bgl_supply -= amount_to_burn;

    BALANCES.save(deps.storage, &balances)?;

    // burn bgl token and send main token
    let msg_list = vec![
        add_funds_to_exec_msg(
            &WasmMsg::Execute {
                contract_addr: minter_address.to_string(),
                msg: to_json_binary(&minter::msg::ExecuteMsg::Burn {})?,
                funds: vec![],
            },
            &[(amount_to_burn, bgl_currency.token)],
        )?
        .into(),
        get_transfer_msg(
            &sender_address,
            amount_to_withdraw,
            &common_config.main_currency.token,
        )?,
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

    if collections.iter().any(|x| x.token_id_list.is_empty()) {
        Err(ContractError::EmptyCollectionList)?;
    }

    // check collection address duplication
    let mut collection_addresses: Vec<String> = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    collection_addresses.sort_unstable();
    collection_addresses.dedup();

    if collections.len() != collection_addresses.len() {
        Err(ContractError::CollectionDuplication)?;
    }

    // check token duplication
    if collections.iter().any(|x| {
        let mut token_list = x.token_id_list.clone();
        token_list.sort_unstable();
        token_list.dedup();

        token_list.len() != x.token_id_list.len()
    }) {
        Err(ContractError::NftDuplication)?;
    }

    // query collection prices
    let block_time = env.block.time.seconds();
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(address_config.oracle, "oracle")?;

    let collection_addresses = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
            start_after: None,
            limit: None,
        },
    )?;

    // check if nfts are on user balance
    check_collections_holder(deps.as_ref(), &sender_address, &collections)?;

    let mut collateral_value = Uint128::zero();
    let mut msg_list: Vec<CosmosMsg> = vec![];

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }

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

        // get collection price
        let oracle::types::PriceItem {
            price,
            price_update_date,
            ..
        } = prices
            .iter()
            .find(|x| x.address == collection_address)
            .ok_or(ContractError::PriceIsNotFound {
                val: collection_address.to_string(),
            })?;

        if block_time - price_update_date > common_config.price_update_period {
            Err(ContractError::OutdatedPrice {
                val: collection_address.to_string(),
            })?;
        }

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
                                if collateral.owner == sender_address {
                                    collateral.token_id_list =
                                        vec![collateral.token_id_list, token_id_list.clone()]
                                            .concat();
                                    collateral.token_id_list.sort_unstable();
                                    collateral.token_id_list.dedup();

                                    // update collateral value
                                    collateral_value += price.amount
                                        * Uint128::new(collateral.token_id_list.len() as u128);

                                    collateral
                                } else {
                                    collateral
                                }
                            })
                            .collect();

                        collaterals
                    }
                    None => {
                        // update collateral value
                        collateral_value +=
                            price.amount * Uint128::new(token_id_list.len() as u128);

                        vec![Collateral {
                            owner: sender_address.clone(),
                            token_id_list,
                        }]
                    }
                };

                Ok(collaterals)
            },
        )?;
    }

    // check collateral value
    if collateral_value < common_config.collateral_min_value {
        Err(ContractError::LowCollateral {
            provided_amount: collateral_value.u128(),
            min_amount: common_config.collateral_min_value.u128(),
            denom: match common_config.main_currency.token {
                Token::Native { denom } => denom,
                Token::Cw20 { address } => address.to_string(),
            },
        })?;
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

    if collections.iter().any(|x| x.token_id_list.is_empty()) {
        Err(ContractError::EmptyCollectionList)?;
    }

    // check collection address duplication
    let mut collection_addresses: Vec<String> = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    collection_addresses.sort_unstable();
    collection_addresses.dedup();

    if collections.len() != collection_addresses.len() {
        Err(ContractError::CollectionDuplication)?;
    }

    // check token duplication
    if collections.iter().any(|x| {
        let mut token_list = x.token_id_list.clone();
        token_list.sort_unstable();
        token_list.dedup();

        token_list.len() != x.token_id_list.len()
    }) {
        Err(ContractError::NftDuplication)?;
    }

    // query collection prices
    let block_time = env.block.time.seconds();
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(address_config.oracle, "oracle")?;

    let collection_addresses = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
            start_after: None,
            limit: None,
        },
    )?;

    // check if nfts are on contract balance
    check_collections_holder(deps.as_ref(), &env.contract.address, &collections)?;

    let mut is_collateral_withdrawn_completely = true;
    let mut collateral_value = Uint128::zero();
    let mut msg_list: Vec<CosmosMsg> = vec![];

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }

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

        // get collection price
        let oracle::types::PriceItem {
            price,
            price_update_date,
            ..
        } = prices
            .iter()
            .find(|x| x.address == collection_address)
            .ok_or(ContractError::PriceIsNotFound {
                val: collection_address.to_string(),
            })?;

        if block_time - price_update_date > common_config.price_update_period {
            Err(ContractError::OutdatedPrice {
                val: collection_address.to_string(),
            })?;
        }

        // update collaterals
        let collaterals: Vec<Collateral> = COLLATERALS
            .load(deps.storage, &collection_address)?
            .into_iter()
            .map(|mut collateral| {
                if collateral.owner == sender_address {
                    collateral
                        .token_id_list
                        .retain(|id| !token_id_list.contains(id));

                    // update collateral value
                    collateral_value +=
                        price.amount * Uint128::new(collateral.token_id_list.len() as u128);

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
            is_collateral_withdrawn_completely = false;
        }
    }

    // check collateral value
    if collateral_value < common_config.collateral_min_value && !is_collateral_withdrawn_completely
    {
        Err(ContractError::LowCollateral {
            provided_amount: collateral_value.u128(),
            min_amount: common_config.collateral_min_value.u128(),
            denom: match common_config.main_currency.token {
                Token::Native { denom } => denom,
                Token::Cw20 { address } => address.to_string(),
            },
        })?;
    }

    // check ltv
    let ltv =
        query::query_conditional_ltv(deps.as_ref(), env, sender_address.to_string(), None, None)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;

    let ltv_max = math::calc_ltv_max(
        rate_config.bid_min_rate,
        rate_config.discount_max_rate,
        rate_config.discount_min_rate,
    );

    if ltv > ltv_max {
        Err(ContractError::ExceededLtv)?;
    }

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "try_withdraw_collateral")]))
}

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
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;

    BORROWERS.update(deps.storage, &sender_address, |x| -> StdResult<Borrower> {
        let mut borrower = x.unwrap_or_default();

        borrower.accumulated_loan = math::calc_accumulated_loan(
            rate_config.borrow_apr,
            borrower.accumulated_loan,
            borrower.loan.amount,
            block_time,
            borrower.loan.creation_date,
        );
        borrower.loan.creation_date = block_time;
        borrower.loan.amount += amount;

        Ok(borrower)
    })?;

    // check ltv
    let ltv =
        query::query_conditional_ltv(deps.as_ref(), env, sender_address.to_string(), None, None)?;
    let ltv_max = math::calc_ltv_max(
        rate_config.bid_min_rate,
        rate_config.discount_max_rate,
        rate_config.discount_min_rate,
    );

    if ltv > ltv_max {
        Err(ContractError::ExceededLtv)?;
    }

    // update states
    accumulate_expected_rewards(deps.storage, block_time)?;

    let mut balances = BALANCES.load(deps.storage)?;

    // don't allow to borrow if available liquidity is too low
    let borrowers_reserve_fraction = math::calc_borrowers_reserve_fraction(
        balances.reserve.deposited,
        common_config.borrowers_reserve_fraction_ratio,
    );

    let available_to_borrow_regular_liquidity = math::calc_available_to_borrow_regular_liquidity(
        balances.regular.deposited,
        balances.regular.accumulated_rewards,
        balances.regular.unbonded,
        balances.regular.borrowed.amount,
    );

    let available_to_borrow_reserve_liquidity = math::calc_available_to_borrow_reserve_liquidity(
        borrowers_reserve_fraction,
        balances.reserve.borrowed.amount,
    );

    let total_available_to_borrow_liquidity =
        available_to_borrow_regular_liquidity + available_to_borrow_reserve_liquidity;

    if amount > total_available_to_borrow_liquidity {
        Err(ContractError::LowAvailableLiquidity)?;
    }

    if amount <= available_to_borrow_regular_liquidity {
        balances.regular.borrowed.amount += amount;
    } else {
        balances.regular.borrowed.amount += available_to_borrow_regular_liquidity;
        balances.reserve.borrowed.amount += amount - available_to_borrow_regular_liquidity;
    }

    BALANCES.save(deps.storage, &balances)?;

    // create send msg
    let msg = get_transfer_msg(&sender_address, amount, &common_config.main_currency.token)?;

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
    let mut response = Response::new().add_attribute("action", "try_repay");

    check_lockout(deps.as_ref())?;
    let (sender_address, asset_amount, asset_info) =
        check_funds(deps.as_ref(), &info, FundsType::Single { sender, amount })?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != common_config.main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // accumulate loan
    let mut borrower = BORROWERS.load(deps.storage, &sender_address)?;
    borrower.accumulated_loan = math::calc_accumulated_loan(
        rate_config.borrow_apr,
        borrower.accumulated_loan,
        borrower.loan.amount,
        block_time,
        borrower.loan.creation_date,
    );
    borrower.loan.creation_date = block_time;

    // try send residue
    let total_loan = borrower.accumulated_loan + borrower.loan.amount;
    let residue = std::cmp::max(asset_amount, total_loan) - total_loan;
    let mut amount_to_repay = asset_amount - residue;

    if !residue.is_zero() {
        response = response.add_message(get_transfer_msg(
            &sender_address,
            residue,
            &common_config.main_currency.token,
        )?);
    }

    // collect platform fee
    let accumulated_loan_to_repay = std::cmp::min(amount_to_repay, borrower.accumulated_loan);
    let platform_revenue = math::calc_platform_revenue_on_repay(
        accumulated_loan_to_repay,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
    );
    increase_platform_revenue(
        deps.storage,
        platform_revenue,
        &common_config.main_currency.token,
        &common_config.main_currency,
    )?;

    // update borrower loan values
    if amount_to_repay <= borrower.accumulated_loan {
        borrower.accumulated_loan -= amount_to_repay;
    } else {
        borrower.loan.amount -= amount_to_repay - borrower.accumulated_loan;
        borrower.accumulated_loan = Uint128::zero();
    }

    // update states
    if borrower.accumulated_loan.is_zero() && borrower.loan.amount.is_zero() {
        BORROWERS.remove(deps.storage, &sender_address);
    } else {
        BORROWERS.save(deps.storage, &sender_address, &borrower)?;
    }

    accumulate_expected_rewards(deps.storage, block_time)?;

    BALANCES.update(deps.storage, |mut balances| -> StdResult<Balances> {
        amount_to_repay -= platform_revenue;

        let accumulated_rewards_diff =
            std::cmp::min(amount_to_repay, balances.regular.accumulated_rewards);
        balances.regular.accumulated_rewards -= accumulated_rewards_diff;
        balances.regular.main_pool += accumulated_rewards_diff;
        amount_to_repay -= accumulated_rewards_diff;

        if amount_to_repay <= balances.reserve.withdrawn {
            balances.reserve.withdrawn -= amount_to_repay;

            return Ok(balances);
        }

        // amount_to_repay -= balances.reserve.withdrawn;
        balances.reserve.withdrawn = Uint128::zero();

        if amount_to_repay <= balances.reserve.borrowed.amount {
            balances.reserve.borrowed.amount -= amount_to_repay;

            return Ok(balances);
        }

        amount_to_repay -= balances.reserve.borrowed.amount;
        balances.reserve.borrowed.amount = Uint128::zero();
        balances.regular.borrowed.amount -= amount_to_repay;

        Ok(balances)
    })?;

    Ok(response)
}

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
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != common_config.main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    if collections.iter().any(|x| x.token_id_list.is_empty()) {
        Err(ContractError::EmptyCollectionList)?;
    }

    // check collection address duplication
    let mut collection_addresses: Vec<String> = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    collection_addresses.sort_unstable();
    collection_addresses.dedup();

    if collections.len() != collection_addresses.len() {
        Err(ContractError::CollectionDuplication)?;
    }

    // limit max token amount
    if collections
        .iter()
        .any(|x| x.token_id_list.len() > LIQUIDATION_BID_MAX_TOKEN_AMOUNT)
    {
        Err(ContractError::ExceededLiquidationBidNftMaxAmount)?;
    }

    // check token duplication
    if collections.iter().any(|x| {
        let mut token_list = x.token_id_list.clone();
        token_list.sort_unstable();
        token_list.dedup();

        token_list.len() != x.token_id_list.len()
    }) {
        Err(ContractError::NftDuplication)?;
    }

    // verify discount
    if discount < rate_config.discount_min_rate || discount > rate_config.discount_max_rate {
        Err(ContractError::DiscountIsOutOfRange)?;
    }

    let mut bid_amount_by_liquidator: usize = 0;
    let mut collection_addresses: Vec<Addr> = vec![];

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }
        collection_addresses.push(collection_address.clone());

        // check if token_id's are in collateral
        let collateral_tokens = COLLATERALS
            .load(deps.storage, &collection_address)?
            .into_iter()
            .fold(Ok(vec![]), |acc, cur| -> StdResult<Vec<String>> {
                // doesn't allow to place bids on own collateral
                if cur.owner == sender_address {
                    Err(ContractError::Unauthorized)?;
                }

                Ok(vec![acc?, cur.token_id_list].concat())
            })?;

        if token_id_list
            .iter()
            .any(|token_id| !collateral_tokens.contains(token_id))
        {
            Err(ContractError::NftIsNotFound)?;
        }

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
                    is_offer: false,
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

                        // don't allow same date bids
                        if bids_by_liquidator
                            .iter()
                            .any(|bid| bid.creation_date == block_time)
                        {
                            Err(ContractError::SameCreationDate)?;
                        }

                        bid_amount_by_liquidator += bids_by_liquidator.len() + 1;
                        y.push(bid);

                        y
                    }
                    None => vec![bid],
                };

                Ok(bids)
            },
        )?;
    }

    // limit max collection amount
    if bid_amount_by_liquidator > LIQUIDATION_BID_MAX_AMOUNT {
        Err(ContractError::ExceededBidAmount)?;
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
        x.regular.liquidation_bids += asset_amount;

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

    let CommonConfig { main_currency, .. } = COMMON_CONFIG.load(deps.storage)?;

    if collections.iter().any(|x| x.token_id_list.is_empty()) {
        Err(ContractError::EmptyCollectionList)?;
    }

    // check collection address duplication
    let mut collection_addresses: Vec<String> = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    collection_addresses.sort_unstable();
    collection_addresses.dedup();

    if collections.len() != collection_addresses.len() {
        Err(ContractError::CollectionDuplication)?;
    }

    // check token duplication
    if collections.iter().any(|x| {
        let mut token_list = x.token_id_list.clone();
        token_list.sort_unstable();
        token_list.dedup();

        token_list.len() != x.token_id_list.len()
    }) {
        Err(ContractError::NftDuplication)?;
    }

    let mut amount_to_withdraw = Uint128::zero();
    let mut collection_addresses: Vec<Addr> = vec![];
    let mut liquidator = LIQUIDATORS.load(deps.storage, &sender_address)?;

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }
        collection_addresses.push(collection_address.clone());

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

        if liquidation_bids.is_empty() {
            LIQUIDATION_BIDS.remove(deps.storage, &collection_address);
            liquidator
                .collection_addresses
                .retain(|x| x != collection_address);
        } else {
            LIQUIDATION_BIDS.save(deps.storage, &collection_address, &liquidation_bids)?;
        }
    }

    if liquidator.collection_addresses.is_empty() {
        LIQUIDATORS.remove(deps.storage, &sender_address);
    } else {
        LIQUIDATORS.save(deps.storage, &sender_address, &liquidator)?;
    }

    BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
        x.regular.liquidation_bids -= amount_to_withdraw;

        Ok(x)
    })?;

    let msg = get_transfer_msg(&sender_address, amount_to_withdraw, &main_currency.token)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_remove_bid")]))
}

#[allow(clippy::too_many_arguments)]
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
            info.sender,
            Uint128::default(),
            Token::new_native(&String::default()),
        )
    } else {
        Err(ContractError::WrongFundsCombination)?
    };

    check_authorization(deps.as_ref(), &sender_address, AuthType::Any)?;

    let block_time = env.block.time.seconds();
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;

    if asset_info != common_config.main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    if collections.iter().any(|x| x.token_id_list.is_empty()) {
        Err(ContractError::EmptyCollectionList)?;
    }

    // check collection address duplication
    let mut collection_addresses: Vec<String> = collections
        .iter()
        .map(|x| x.collection_address.to_string())
        .collect();

    collection_addresses.sort_unstable();
    collection_addresses.dedup();

    if collections.len() != collection_addresses.len() {
        Err(ContractError::CollectionDuplication)?;
    }

    // limit max token amount
    if collections
        .iter()
        .any(|x| x.token_id_list.len() > LIQUIDATION_BID_MAX_TOKEN_AMOUNT)
    {
        Err(ContractError::ExceededLiquidationBidNftMaxAmount)?;
    }

    // check token duplication
    if collections.iter().any(|x| {
        let mut token_list = x.token_id_list.clone();
        token_list.sort_unstable();
        token_list.dedup();

        token_list.len() != x.token_id_list.len()
    }) {
        Err(ContractError::NftDuplication)?;
    }

    // verify discount
    if discount < rate_config.discount_min_rate || discount > rate_config.discount_max_rate {
        Err(ContractError::DiscountIsOutOfRange)?;
    }

    let mut current_bid_value = Uint128::zero();
    let mut new_bid_value = Uint128::zero();
    let mut collection_addresses: Vec<Addr> = vec![];

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        // validate collection
        let collection_address = deps.api.addr_validate(&collection_address)?;
        if !COLLECTIONS.has(deps.storage, &collection_address) {
            Err(ContractError::CollectionIsNotFound)?;
        }
        collection_addresses.push(collection_address.clone());

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
                                        is_offer: false,
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

    let response = Response::new().add_attributes(vec![("action", "try_update_bid")]);

    if new_bid_value >= current_bid_value && current_bid_value + asset_amount >= new_bid_value {
        let amount_to_return = current_bid_value + asset_amount - new_bid_value;

        BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
            x.regular.liquidation_bids += new_bid_value - current_bid_value;

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
            x.regular.liquidation_bids -= current_bid_value - new_bid_value;

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

pub fn try_liquidate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    targets: Vec<LiquidationItem>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let block_time = env.block.time.seconds();
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let scheduler = unwrap_field(address_config.scheduler, "scheduler")?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(scheduler)],
        },
    )?;

    // accumulators
    let mut total_liquidation_value = Uint128::zero();
    let total_liquidation_bids = Uint128::zero();

    // iterate over borrowers
    for LiquidationItem {
        borrower,
        liquidation_set,
    } in targets
    {
        let borrower_address = deps.api.addr_validate(&borrower)?;
        let Borrower {
            accumulated_loan,
            loan,
        } = BORROWERS.load(deps.storage, &borrower_address)?;

        let current_liquidation_value =
            update_liquidation_storages(deps.storage, &liquidation_set)?;

        // update total_liquidation_value and borrowers_storages
        total_liquidation_value += current_liquidation_value;

        update_borrowers_storages(
            deps.storage,
            &loan,
            &borrower_address,
            current_liquidation_value,
            accumulated_loan,
            block_time,
            rate_config.borrow_apr,
        )?;
    }

    // collect platform fee
    let platform_revenue = math::calc_platform_revenue_on_liquidation(
        total_liquidation_value,
        rate_config.liquidation_fee_rate,
    );
    let main_currency_pool_addition = total_liquidation_value - platform_revenue;

    increase_platform_revenue(
        deps.storage,
        platform_revenue,
        &common_config.main_currency.token,
        &common_config.main_currency,
    )?;

    // update contract balances
    update_balances(
        deps.storage,
        block_time,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
        main_currency_pool_addition,
        total_liquidation_value,
        total_liquidation_bids,
    )?;

    Ok(Response::new().add_attributes(vec![("action", "try_liquidate")]))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_address_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    oracle: Option<String>,
    minter: Option<String>,
    scheduler: Option<String>,
    marketplace: Option<String>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    accumulate_expected_rewards(deps.storage, env.block.time.seconds())?;
    let mut attrs = Attrs::init("try_update_address_config");

    ADDRESS_CONFIG.update(deps.storage, |mut config| -> StdResult<AddressConfig> {
        if let Some(x) = owner {
            config.owner = Some(deps.api.addr_validate(&x)?);
            attrs.push(("owner".to_string(), x));
        }

        if let Some(x) = oracle {
            config.oracle = Some(deps.api.addr_validate(&x)?);
            attrs.push(("oracle".to_string(), x));
        }

        if let Some(x) = minter {
            config.minter = Some(deps.api.addr_validate(&x)?);
            attrs.push(("minter".to_string(), x));
        }

        if let Some(x) = scheduler {
            config.scheduler = Some(deps.api.addr_validate(&x)?);
            attrs.push(("scheduler".to_string(), x));
        }

        if let Some(x) = marketplace {
            config.marketplace = Some(deps.api.addr_validate(&x)?);
            attrs.push(("marketplace".to_string(), x));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_rate_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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

    accumulate_expected_rewards(deps.storage, env.block.time.seconds())?;
    let mut attrs = Attrs::init("try_update_rate_config");

    RATE_CONFIG.update(deps.storage, |mut config| -> StdResult<RateConfig> {
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

#[allow(clippy::too_many_arguments)]
pub fn try_update_common_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    main_currency: Option<Currency<TokenUnverified>>,
    bgl_currency: Option<Currency<TokenUnverified>>,
    unbonding_period: Option<u64>,
    collateral_min_value: Option<Uint128>,
    borrowers_reserve_fraction_ratio: Option<Decimal>,
    price_update_period: Option<u64>,
    is_marketplace_interface_enabled: Option<bool>,
) -> Result<Response, ContractError> {
    check_lockout(deps.as_ref())?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;

    accumulate_expected_rewards(deps.storage, env.block.time.seconds())?;
    let balances = BALANCES.load(deps.storage)?;
    let mut attrs = Attrs::init("try_update_common_config");

    COMMON_CONFIG.update(deps.storage, |mut config| -> StdResult<CommonConfig> {
        if let Some(x) = main_currency {
            if !balances.regular.main_pool.is_zero() {
                Err(ContractError::ChangingCurrency)?;
            }

            config.main_currency = Currency::new(&x.token.verify(deps.api)?, x.decimals);
            attrs.push(("main_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = bgl_currency {
            if !balances.regular.bgl_supply.is_zero() {
                Err(ContractError::ChangingCurrency)?;
            }

            config.bgl_currency = Some(Currency::new(&x.token.verify(deps.api)?, x.decimals));
            attrs.push(("bgl_currency".to_string(), x.token.get_symbol()));
        }

        if let Some(x) = unbonding_period {
            config.unbonding_period = x;
            attrs.push(("unbonding_period".to_string(), x.to_string()));
        }

        if let Some(x) = collateral_min_value {
            config.collateral_min_value = x;
            attrs.push(("collateral_min_value".to_string(), x.to_string()));
        }

        if let Some(x) = borrowers_reserve_fraction_ratio {
            config.borrowers_reserve_fraction_ratio = x;
            attrs.push((
                "borrowers_reserve_fraction_ratio".to_string(),
                x.to_string(),
            ));
        }

        if let Some(x) = price_update_period {
            config.price_update_period = x;
            attrs.push(("price_update_period".to_string(), x.to_string()));
        }

        if let Some(x) = is_marketplace_interface_enabled {
            config.is_marketplace_interface_enabled = x;
            attrs.push((
                "is_marketplace_interface_enabled".to_string(),
                x.to_string(),
            ));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
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
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let bgl_currency = unwrap_field(common_config.bgl_currency, "bgl_currency")?;
    let minter_address = unwrap_field(address_config.minter, "minter")?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != common_config.main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update contract balances
    let mut balances = BALANCES.load(deps.storage)?;

    let expected_rewards = math::calc_expected_rewards(
        block_time,
        balances.regular.borrowed.creation_date,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
        balances.regular.borrowed.amount,
    );
    let rewards = balances.regular.accumulated_rewards + expected_rewards;

    let bgl_token_amount = math::calc_minted_bgl_token_amount(
        asset_amount,
        balances.regular.main_pool,
        balances.regular.bgl_supply,
        rewards,
        common_config.main_currency.decimals,
        bgl_currency.decimals,
    );

    // it doesn't affect on deposit APR
    balances.reserve.deposited += asset_amount;
    balances.reserve.bgl_supply += bgl_token_amount;

    BALANCES.save(deps.storage, &balances)?;

    // mint bgl_token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: minter_address.to_string(),
        msg: to_json_binary(&minter::msg::ExecuteMsg::Mint {
            token: bgl_currency.token.into(),
            amount: bgl_token_amount,
            recipient: sender_address.to_string(),
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_deposit_reserve_liquidity")]))
}

pub fn try_reinforce_bgl_token(
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

    let CommonConfig { main_currency, .. } = COMMON_CONFIG.load(deps.storage)?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    if asset_info != main_currency.token {
        Err(ContractError::WrongAssetType)?;
    }

    // update contract balances, it doesn't affect on deposit APR and bgl token supply
    BALANCES.update(deps.storage, |mut x| -> StdResult<Balances> {
        x.regular.main_pool += asset_amount;

        Ok(x)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "try_reinforce_bgl_token")]))
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
    increase_platform_revenue(
        deps.storage,
        asset_amount,
        &asset_info,
        &proposal.listing_price.currency,
    )?;

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
