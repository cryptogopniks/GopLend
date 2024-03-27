use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, StdResult, Uint128};

use goplend_base::{
    assets::{Funds, Token},
    error::ContractError,
    lending_platform::{
        msg::{
            QueryBorrowersResponseItem, QueryCollateralsResponseItem, QueryCollectionsResponseItem,
            QueryLiquidationBidsResponseItem, QueryLiquidatorsResponseItem,
            QueryProposalsResponseItem, QueryUnbondersResponseItem,
        },
        state::{
            BALANCES, BORROWERS, COLLATERALS, COLLECTIONS, CONFIG, LIQUIDATION_BIDS, LIQUIDATORS,
            PLATFORM_REVENUE, PROPOSALS, UNBONDERS,
        },
        types::{
            Balances, Borrower, Collateral, Collection, Config, LiquidationBid, LiquidationItem,
            Liquidator, Unbonder,
        },
    },
    oracle,
    utils::{filter_by_address_list, unwrap_field},
};

use crate::{
    helpers::{
        get_borrowers, get_collateral_value, get_collaterals_by_borrower,
        get_collection_and_bid_list, get_collection_prices,
    },
    math::{self, match_collaterals_and_bids},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_platform_revenue(deps: Deps, _env: Env) -> StdResult<Vec<Funds<Token>>> {
    PLATFORM_REVENUE.load(deps.storage)
}

pub fn query_balances(deps: Deps, _env: Env) -> StdResult<Balances> {
    BALANCES.load(deps.storage)
}

pub fn query_unbonders(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryUnbondersResponseItem>> {
    let depositors: Vec<(Addr, Unbonder)> = UNBONDERS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(filter_by_address_list(deps.api, &addresses, &depositors)?
        .into_iter()
        .map(|(address, unbonder)| QueryUnbondersResponseItem { address, unbonder })
        .collect())
}

pub fn query_borrowers(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryBorrowersResponseItem>> {
    let borrowers: Vec<(Addr, Borrower)> = BORROWERS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(filter_by_address_list(deps.api, &addresses, &borrowers)?
        .into_iter()
        .map(|(address, borrower)| QueryBorrowersResponseItem { address, borrower })
        .collect())
}

pub fn query_liquidators(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryLiquidatorsResponseItem>> {
    let liquidators: Vec<(Addr, Liquidator)> = LIQUIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(filter_by_address_list(deps.api, &addresses, &liquidators)?
        .into_iter()
        .map(|(address, liquidator)| QueryLiquidatorsResponseItem {
            address,
            liquidator,
        })
        .collect())
}

pub fn query_collaterals(
    deps: Deps,
    _env: Env,
    collection_addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryCollateralsResponseItem>> {
    let collaterals: Vec<(Addr, Vec<Collateral>)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(
        filter_by_address_list(deps.api, &collection_addresses, &collaterals)?
            .into_iter()
            .map(|(address, collateral)| QueryCollateralsResponseItem {
                address,
                collateral,
            })
            .collect(),
    )
}

pub fn query_liquidation_bids_by_collection_addresses(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryLiquidationBidsResponseItem>> {
    let liquidation_bids: Vec<(Addr, Vec<LiquidationBid>)> = LIQUIDATION_BIDS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(
        filter_by_address_list(deps.api, &addresses, &liquidation_bids)?
            .into_iter()
            .map(
                |(address, liquidation_bids)| QueryLiquidationBidsResponseItem {
                    address,
                    liquidation_bids,
                },
            )
            .collect(),
    )
}

pub fn query_liquidation_bids_by_liquidator_addresses(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryLiquidationBidsResponseItem>> {
    let liquidators: Vec<(Addr, Liquidator)> = LIQUIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    let liquidation_bids: Vec<QueryLiquidationBidsResponseItem> =
        filter_by_address_list(deps.api, &addresses, &liquidators)?
            .into_iter()
            .map(
                |(
                    liquidator_address,
                    Liquidator {
                        collection_addresses,
                    },
                )|
                 -> StdResult<QueryLiquidationBidsResponseItem> {
                    let liquidation_bids = collection_addresses
                        .into_iter()
                        .map(|collection_address| -> StdResult<Vec<LiquidationBid>> {
                            LIQUIDATION_BIDS.load(deps.storage, &collection_address)
                        })
                        .collect::<StdResult<Vec<Vec<LiquidationBid>>>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<LiquidationBid>>();

                    Ok(QueryLiquidationBidsResponseItem {
                        address: liquidator_address,
                        liquidation_bids,
                    })
                },
            )
            .collect::<StdResult<Vec<QueryLiquidationBidsResponseItem>>>()?;

    Ok(liquidation_bids)
}

pub fn query_proposals(
    deps: Deps,
    _env: Env,
    last_amount: Option<u32>,
) -> StdResult<Vec<QueryProposalsResponseItem>> {
    let proposals: Vec<QueryProposalsResponseItem> = PROPOSALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .map(|(id, proposal)| QueryProposalsResponseItem { id, proposal })
        .collect();

    if let Some(x) = last_amount {
        let x = (x as usize).clamp(0, proposals.len());
        let from = proposals.len() - x;
        return Ok(proposals[from..].to_vec());
    }

    Ok(proposals)
}

pub fn query_collections(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryCollectionsResponseItem>> {
    let collections: Vec<(Addr, Collection<Addr>)> = COLLECTIONS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(filter_by_address_list(deps.api, &addresses, &collections)?
        .into_iter()
        .map(|(address, collection)| QueryCollectionsResponseItem {
            address,
            collection,
        })
        .collect())
}

pub fn query_bgl_currency_to_main_currency_price(deps: Deps, env: Env) -> StdResult<Decimal> {
    let block_time = env.block.time.seconds();
    let balances = BALANCES.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let main_currency = config.main_currency;
    let bgl_currency = unwrap_field(config.bgl_currency, "bgl_currency")?;

    let expected_rewards = math::calc_expected_rewards(
        block_time,
        balances.total_borrowed.creation_date,
        config.borrow_apr,
        balances.total_borrowed.amount,
    );
    let rewards = balances.accumulated_expected_rewards + expected_rewards;

    Ok(math::calc_bgl_to_main_price(
        balances.total_main_currency_pool,
        balances.total_bgl_currency_supply,
        rewards,
        main_currency.decimals,
        bgl_currency.decimals,
    ))
}

pub fn query_conditional_deposit_apr(
    deps: Deps,
    _env: Env,
    amount_to_deposit: Option<Uint128>,
) -> StdResult<Decimal> {
    let Config {
        borrow_apr,
        borrow_fee_rate,
        ..
    } = CONFIG.load(deps.storage)?;
    let Balances {
        total_deposited,
        total_unbonded,
        total_borrowed,
        ..
    } = BALANCES.load(deps.storage)?;

    Ok(math::calc_conditional_deposit_apr(
        borrow_apr,
        borrow_fee_rate,
        amount_to_deposit.unwrap_or_default(),
        total_deposited,
        total_unbonded,
        total_borrowed.amount,
    ))
}

pub fn query_conditional_ltv(
    deps: Deps,
    env: Env,
    borrower: String,
    amount_to_deposit: Option<Uint128>,
    amount_to_borrow: Option<Uint128>,
) -> StdResult<Decimal> {
    let block_time = env.block.time.seconds();
    let Config {
        borrow_apr,
        oracle,
        price_update_period,
        ..
    } = CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(oracle, "oracle")?;
    let borrower_address = deps.api.addr_validate(&borrower)?;
    let borrower = BORROWERS
        .load(deps.storage, &borrower_address)
        .unwrap_or_default();

    // get collaterals owned by borrower
    let collection_and_collateral_list: Vec<(Addr, Collateral)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .filter(|(_collection, collaterals)| {
            collaterals
                .into_iter()
                .any(|collateral| collateral.owner == borrower_address)
        })
        .map(
            |(collection, collaterals)| -> StdResult<(Addr, Collateral)> {
                // single collateral per collection
                let collateral = collaterals
                    .into_iter()
                    .find(|collateral| collateral.owner == borrower_address)
                    .ok_or(ContractError::CollateralIsNotFound)?;

                Ok((collection, collateral))
            },
        )
        .collect::<StdResult<Vec<(Addr, Collateral)>>>()?;

    let collection_addresses: Vec<String> = collection_and_collateral_list
        .iter()
        .map(|(collection, _collateral)| collection.to_string())
        .collect();

    // TODO: limit prices change rate to protect against oracle attack
    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
        },
    )?;

    let collateral_value = collection_and_collateral_list.iter().fold(
        Ok(Uint128::zero()),
        |acc, (collection_address, collateral)| -> StdResult<Uint128> {
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

            if block_time - price_update_date > price_update_period {
                Err(ContractError::OutdatedPrice {
                    val: collection_address.to_string(),
                })?;
            }

            // TODO: decimals, other currencies
            Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
        },
    )?;

    if collateral_value.is_zero()
        && !(borrower.accumulated_loan.is_zero() && borrower.loan.amount.is_zero())
    {
        Err(ContractError::CollateralIsNotFound)?;
    }

    Ok(math::calc_conditional_ltv(
        borrow_apr,
        amount_to_borrow.unwrap_or_default(),
        borrower.accumulated_loan,
        borrower.loan.amount,
        amount_to_deposit.unwrap_or_default(),
        collateral_value,
        block_time,
        borrower.loan.creation_date,
    ))
}

pub fn query_total_available_to_borrow_liquidity(deps: Deps, env: Env) -> StdResult<Uint128> {
    let block_time = env.block.time.seconds();
    let config = CONFIG.load(deps.storage)?;
    let balances = BALANCES.load(deps.storage)?;

    let expected_rewards = math::calc_expected_rewards(
        block_time,
        balances.total_borrowed.creation_date,
        config.borrow_apr,
        balances.total_borrowed.amount,
    );
    let rewards = balances.accumulated_expected_rewards + expected_rewards;

    let min_available_liquidity = math::calc_min_available_liquidity(
        config.min_available_liquidity_multiplier,
        balances.total_main_currency_pool,
        balances.reserve_main_currency_pool,
        rewards,
    );

    let total_available_to_borrow_liquidity = math::calc_total_available_to_borrow_liquidity(
        balances.total_main_currency_pool,
        balances.total_borrowed.amount,
        min_available_liquidity,
    );

    Ok(total_available_to_borrow_liquidity)
}

pub fn query_targets(
    deps: Deps,
    env: Env,
    borrowers: Vec<String>,
) -> StdResult<Vec<LiquidationItem>> {
    let block_time = env.block.time.seconds();
    let Config {
        borrow_apr,
        oracle,
        discount_max_rate,
        discount_min_rate,
        bid_min_rate,
        price_update_period,
        ..
    } = CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(oracle, "oracle")?;

    let borrowers = get_borrowers(deps.storage, deps.api, &borrowers)?;

    // get all collaterals
    let collection_and_collateral_list: Vec<(Addr, Vec<Collateral>)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    // get collection prices
    let prices = get_collection_prices(deps, &borrowers, &collection_and_collateral_list, oracle)?;

    // accumulators
    let mut targets: Vec<LiquidationItem> = vec![];

    // iterate over borrowers
    for (
        borrower_address,
        Borrower {
            accumulated_loan,
            loan,
        },
    ) in borrowers
    {
        // get collaterals owned by borrower
        let collection_and_collateral_list_by_borrower =
            get_collaterals_by_borrower(&collection_and_collateral_list, &borrower_address)?;

        let collateral_value_option = get_collateral_value(
            &collection_and_collateral_list_by_borrower,
            &prices,
            block_time,
            price_update_period,
        )?;
        // go to next borrower if any price is outdated
        if collateral_value_option.is_none() {
            continue;
        }
        let collateral_value = collateral_value_option.unwrap();

        // check ltv
        let ltv_max = math::calc_ltv_max(bid_min_rate, discount_max_rate, discount_min_rate);
        let ltv = math::calc_conditional_ltv(
            borrow_apr,
            Uint128::zero(),
            accumulated_loan,
            loan.amount,
            Uint128::zero(),
            collateral_value,
            block_time,
            loan.creation_date,
        );

        // if ltv is low check next borrower, otherwise liquidate required amount
        if ltv <= ltv_max {
            continue;
        }

        // find bids containing at least 1 borrower's collateral token
        let collection_and_bid_list =
            get_collection_and_bid_list(deps.storage, &collection_and_collateral_list_by_borrower)?;

        let bidded_collateral_list = match_collaterals_and_bids(
            &collection_and_collateral_list_by_borrower,
            &collection_and_bid_list,
            &prices,
        )?;

        // calculate how much tokens and what collection must be liquidated to get proper ltv
        // for worst case (max discount)
        let max_liquidation_value =
            math::calc_liquidation_value(collateral_value, discount_max_rate, ltv, ltv_max);

        let liquidation_set = math::calc_liquidation_set(
            &bidded_collateral_list,
            max_liquidation_value,
            ltv_max,
            borrow_apr,
            accumulated_loan,
            loan.amount,
            collateral_value,
            block_time,
            loan.creation_date,
        );

        targets.push(LiquidationItem {
            borrower: borrower_address.to_string(),
            liquidation_set,
        });
    }

    Ok(targets)
}
