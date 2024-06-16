use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, StdResult, Uint128};

use cw_storage_plus::Bound;
use goplend_base::{
    assets::{Funds, Token},
    error::ContractError,
    lending_platform::{
        msg::{
            QueryAmountsResponse, QueryBorrowersResponseItem, QueryCollateralsResponseItem,
            QueryCollectionsResponseItem, QueryLiquidationBidsByCollectionAddressListResponseItem,
            QueryLiquidationBidsByLiquidatorAddressListResponseItem, QueryLiquidatorsResponseItem,
            QueryProposalsResponseItem, QueryUnbondersResponseItem,
        },
        state::{
            ADDRESS_CONFIG, BALANCES, BORROWERS, COLLATERALS, COLLECTIONS, COMMON_CONFIG,
            LIQUIDATION_BIDS, LIQUIDATORS, PAGINATION_DEFAULT_LIMIT, PAGINATION_MAX_LIMIT,
            PLATFORM_REVENUE, PRICE_UPDATE_PERIOD, PROPOSALS, RATE_CONFIG, UNBONDERS,
        },
        types::{
            AddressConfig, Balances, Borrower, Collateral, Collection, CommonConfig,
            LiquidationBid, Liquidator, RateConfig, Unbonder,
        },
    },
    oracle,
    utils::unwrap_field,
};

use crate::{helpers, math};

pub fn query_address_config(deps: Deps, _env: Env) -> StdResult<AddressConfig> {
    ADDRESS_CONFIG.load(deps.storage)
}

pub fn query_rate_config(deps: Deps, _env: Env) -> StdResult<RateConfig> {
    RATE_CONFIG.load(deps.storage)
}

pub fn query_common_config(deps: Deps, _env: Env) -> StdResult<CommonConfig> {
    COMMON_CONFIG.load(deps.storage)
}

pub fn query_platform_revenue(deps: Deps, _env: Env) -> StdResult<Vec<Funds<Token>>> {
    PLATFORM_REVENUE.load(deps.storage)
}

pub fn query_balances(deps: Deps, env: Env) -> StdResult<Balances> {
    helpers::get_balances_with_updated_rewards(deps.storage, env.block.time.seconds())
}

pub fn query_unbonder_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryUnbondersResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(UNBONDERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (address, unbonder) = x.unwrap();
            QueryUnbondersResponseItem { address, unbonder }
        })
        .collect())
}

pub fn query_unbonder(deps: Deps, _env: Env, address: String) -> StdResult<Unbonder> {
    Ok(UNBONDERS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default())
}

pub fn query_borrower_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryBorrowersResponseItem>> {
    let block_time_ceiled =
        math::calc_block_time_ceiled(env.block.time.seconds(), PRICE_UPDATE_PERIOD);
    let RateConfig { borrow_apr, .. } = RATE_CONFIG.load(deps.storage)?;

    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(BORROWERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (address, mut borrower) = x.unwrap();

            borrower.accumulated_loan = math::calc_accumulated_loan(
                borrow_apr,
                borrower.accumulated_loan,
                borrower.loan.amount,
                block_time_ceiled,
                borrower.loan.creation_date,
            );

            QueryBorrowersResponseItem { address, borrower }
        })
        .collect())
}

pub fn query_borrower(deps: Deps, env: Env, address: String) -> StdResult<Borrower> {
    let mut borrower = BORROWERS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default();

    borrower.accumulated_loan = math::calc_accumulated_loan(
        RATE_CONFIG.load(deps.storage)?.borrow_apr,
        borrower.accumulated_loan,
        borrower.loan.amount,
        math::calc_block_time_ceiled(env.block.time.seconds(), PRICE_UPDATE_PERIOD),
        borrower.loan.creation_date,
    );

    Ok(borrower)
}

pub fn query_liquidator_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryLiquidatorsResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(LIQUIDATORS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (address, liquidator) = x.unwrap();
            QueryLiquidatorsResponseItem {
                address,
                liquidator,
            }
        })
        .collect())
}

pub fn query_liquidator(deps: Deps, _env: Env, address: String) -> StdResult<Liquidator> {
    Ok(LIQUIDATORS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default())
}

pub fn query_collateral_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryCollateralsResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(COLLATERALS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (address, collateral) = x.unwrap();
            QueryCollateralsResponseItem {
                address,
                collateral,
            }
        })
        .collect())
}

pub fn query_collateral(deps: Deps, _env: Env, address: String) -> StdResult<Vec<Collateral>> {
    Ok(COLLATERALS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default())
}

pub fn query_collateral_by_owner(
    deps: Deps,
    _env: Env,
    owner: String,
) -> StdResult<Vec<QueryCollateralsResponseItem>> {
    Ok(COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .filter(|(_, collateral)| collateral.iter().any(|x| x.owner == owner))
        .map(|(address, collateral)| QueryCollateralsResponseItem {
            address,
            collateral: collateral
                .into_iter()
                .filter(|x| x.owner == owner)
                .collect(),
        })
        .collect())
}

pub fn query_liquidation_bids_by_collection_address_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(LIQUIDATION_BIDS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (address, liquidation_bids) = x.unwrap();
            QueryLiquidationBidsByCollectionAddressListResponseItem {
                collection_address: address,
                liquidation_bids,
            }
        })
        .collect())
}

pub fn query_liquidation_bids_by_collection_address(
    deps: Deps,
    _env: Env,
    address: String,
) -> StdResult<Vec<LiquidationBid>> {
    Ok(LIQUIDATION_BIDS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default())
}

pub fn query_liquidation_bids_by_liquidator_address_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    LIQUIDATORS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (
                liquidator_address,
                Liquidator {
                    collection_addresses,
                },
            ) = x.unwrap();

            let collection_and_liquidation_bids_list = collection_addresses
                .into_iter()
                .map(|collection_address| {
                    let liquidation_bids =
                        LIQUIDATION_BIDS.load(deps.storage, &collection_address)?;

                    Ok(QueryLiquidationBidsByCollectionAddressListResponseItem {
                        collection_address,
                        liquidation_bids,
                    })
                })
                .collect::<StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>>(
                )?;

            Ok(QueryLiquidationBidsByLiquidatorAddressListResponseItem {
                liquidator_address,
                collection_and_liquidation_bids_list,
            })
        })
        .collect::<StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>>>()
}

pub fn query_liquidation_bids_by_liquidator_address(
    deps: Deps,
    _env: Env,
    address: String,
) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
    let liquidator = LIQUIDATORS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default();

    liquidator
        .collection_addresses
        .into_iter()
        .map(|collection_address| {
            let liquidation_bids = LIQUIDATION_BIDS.load(deps.storage, &collection_address)?;

            Ok(QueryLiquidationBidsByCollectionAddressListResponseItem {
                collection_address,
                liquidation_bids,
            })
        })
        .collect::<StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>>()
}

pub fn query_proposals(
    deps: Deps,
    _env: Env,
    start_after: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryProposalsResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    Ok(PROPOSALS
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|x| {
            let (id, proposal) = x.unwrap();
            QueryProposalsResponseItem { id, proposal }
        })
        .collect())
}

pub fn query_collection_list(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryCollectionsResponseItem>> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(COLLECTIONS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|x| {
            let (address, collection) = x.unwrap();
            QueryCollectionsResponseItem {
                address,
                collection,
            }
        })
        .collect())
}

pub fn query_collection(deps: Deps, _env: Env, address: String) -> StdResult<Collection<Addr>> {
    COLLECTIONS.load(deps.storage, &deps.api.addr_validate(&address)?)
}

pub fn query_bgl_currency_to_main_currency_price(deps: Deps, env: Env) -> StdResult<Decimal> {
    let block_time = env.block.time.seconds();
    let balances = BALANCES.load(deps.storage)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;

    let regular_rewards = math::calc_expected_rewards(
        block_time,
        balances.regular.borrowed.creation_date,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
        balances.regular.borrowed.amount,
    );

    let reserve_rewards = math::calc_expected_rewards(
        block_time,
        balances.reserve.borrowed.creation_date,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
        balances.reserve.borrowed.amount,
    );

    let rewards = balances.regular.accumulated_rewards + regular_rewards + reserve_rewards;

    Ok(math::calc_bgl_to_main_price(
        balances.regular.main_pool,
        balances.regular.bgl_supply,
        rewards,
    ))
}

pub fn query_conditional_deposit_apr(
    deps: Deps,
    _env: Env,
    amount_to_deposit: Option<Uint128>,
) -> StdResult<Decimal> {
    let RateConfig {
        borrow_apr,
        borrow_fee_rate,
        ..
    } = RATE_CONFIG.load(deps.storage)?;
    let Balances { regular, reserve } = BALANCES.load(deps.storage)?;

    Ok(math::calc_conditional_deposit_apr(
        borrow_apr,
        borrow_fee_rate,
        amount_to_deposit.unwrap_or_default(),
        regular.deposited,
        regular.unbonded,
        regular.borrowed.amount,
        reserve.borrowed.amount,
    ))
}

pub fn query_ltv_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<(Addr, Decimal)>> {
    let block_time = env.block.time.seconds();
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(address_config.oracle, "oracle")?;

    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;

    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: None,
            start_after: None,
            limit: None,
        },
    )?;

    let all_collaterals = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Collateral>)>>>()?;

    BORROWERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .flatten()
        .map(|(borrower_address, borrower)| {
            // get collaterals owned by borrower
            let mut collection_and_collateral_list: Vec<(Addr, Collateral)> = vec![];

            for (collection, collaterals) in &all_collaterals {
                // single collateral per collection
                if let Some(collateral) = collaterals
                    .iter()
                    .find(|collateral| collateral.owner == borrower_address)
                {
                    collection_and_collateral_list
                        .push((collection.to_owned(), collateral.to_owned()));
                }
            }

            let collateral_value = collection_and_collateral_list.iter().fold(
                Ok(Uint128::zero()),
                |acc, (collection_address, collateral)| -> StdResult<Uint128> {
                    let price_item = prices
                        .iter()
                        .find(|x| x.address == collection_address)
                        .ok_or(ContractError::PriceIsNotFound {
                            val: collection_address.to_string(),
                        })?;

                    if block_time - price_item.price_update_date > common_config.price_update_period
                    {
                        Err(ContractError::OutdatedPrice {
                            val: collection_address.to_string(),
                        })?;
                    }

                    let price = helpers::choose_price(&common_config.main_currency, &price_item);

                    Ok(acc? + price * Uint128::new(collateral.token_id_list.len() as u128))
                },
            )?;

            let ltv = math::calc_conditional_ltv(
                rate_config.borrow_apr,
                Uint128::zero(),
                borrower.accumulated_loan,
                borrower.loan.amount,
                Uint128::zero(),
                collateral_value,
                block_time,
                borrower.loan.creation_date,
            );

            Ok((borrower_address, ltv))
        })
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()
}

pub fn query_conditional_ltv(
    deps: Deps,
    env: Env,
    borrower: String,
    amount_to_deposit: Option<Uint128>,
    amount_to_borrow: Option<Uint128>,
) -> StdResult<Decimal> {
    let block_time = env.block.time.seconds();
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(address_config.oracle, "oracle")?;
    let borrower_address = deps.api.addr_validate(&borrower)?;
    let borrower = BORROWERS
        .load(deps.storage, &borrower_address)
        .unwrap_or_default();

    // get collaterals owned by borrower
    let mut collection_and_collateral_list: Vec<(Addr, Collateral)> = vec![];

    for (collection, collaterals) in COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
    {
        if let Some(collateral) = collaterals
            .iter()
            .find(|collateral| collateral.owner == borrower_address)
        {
            collection_and_collateral_list.push((collection, collateral.to_owned()));
        }
    }

    let collection_addresses: Vec<String> = collection_and_collateral_list
        .iter()
        .map(|(collection, _collateral)| collection.to_string())
        .collect();

    // TODO: limit prices change rate to protect against oracle attack
    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
            start_after: None,
            limit: None,
        },
    )?;

    let collateral_value = collection_and_collateral_list.iter().fold(
        Ok(Uint128::zero()),
        |acc, (collection_address, collateral)| -> StdResult<Uint128> {
            let price_item = prices
                .iter()
                .find(|x| x.address == collection_address)
                .ok_or(ContractError::PriceIsNotFound {
                    val: collection_address.to_string(),
                })?;

            if block_time - price_item.price_update_date > common_config.price_update_period {
                Err(ContractError::OutdatedPrice {
                    val: collection_address.to_string(),
                })?;
            }

            let price = helpers::choose_price(&common_config.main_currency, &price_item);

            Ok(acc? + price * Uint128::new(collateral.token_id_list.len() as u128))
        },
    )?;

    if collateral_value.is_zero()
        && !(borrower.accumulated_loan.is_zero() && borrower.loan.amount.is_zero())
    {
        return Ok(Decimal::zero());
    }

    Ok(math::calc_conditional_ltv(
        rate_config.borrow_apr,
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
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let balances = BALANCES.load(deps.storage)?;

    let regular_rewards = math::calc_expected_rewards(
        block_time,
        balances.regular.borrowed.creation_date,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
        balances.regular.borrowed.amount,
    );

    let reserve_rewards = math::calc_expected_rewards(
        block_time,
        balances.reserve.borrowed.creation_date,
        rate_config.borrow_apr,
        rate_config.borrow_fee_rate,
        balances.reserve.borrowed.amount,
    );

    let rewards = balances.regular.accumulated_rewards + regular_rewards + reserve_rewards;

    let borrowers_reserve_fraction = math::calc_borrowers_reserve_fraction(
        balances.reserve.deposited,
        common_config.borrowers_reserve_fraction_ratio,
    );

    let total_available_to_borrow_liquidity = math::calc_total_available_to_borrow_liquidity(
        balances.regular.deposited,
        rewards,
        balances.regular.unbonded,
        balances.regular.borrowed.amount,
        borrowers_reserve_fraction,
        balances.reserve.borrowed.amount,
    );

    Ok(total_available_to_borrow_liquidity)
}

pub fn query_available_to_borrow(
    deps: Deps,
    env: Env,
    borrower: String,
    target_ltv: Option<Decimal>,
) -> StdResult<Uint128> {
    let block_time = env.block.time.seconds();
    let address_config = ADDRESS_CONFIG.load(deps.storage)?;
    let rate_config = RATE_CONFIG.load(deps.storage)?;
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(address_config.oracle, "oracle")?;
    let borrower_address = deps.api.addr_validate(&borrower)?;
    let borrower = BORROWERS
        .load(deps.storage, &borrower_address)
        .unwrap_or_default();

    // get collaterals owned by borrower
    let mut collection_and_collateral_list: Vec<(Addr, Collateral)> = vec![];

    for (collection, collaterals) in COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
    {
        // single collateral per collection
        if let Some(collateral) = collaterals
            .iter()
            .find(|collateral| collateral.owner == borrower_address)
        {
            collection_and_collateral_list.push((collection.to_owned(), collateral.to_owned()));
        }
    }

    let collection_addresses: Vec<String> = collection_and_collateral_list
        .iter()
        .map(|(collection, _collateral)| collection.to_string())
        .collect();

    // TODO: limit prices change rate to protect against oracle attack
    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
            start_after: None,
            limit: None,
        },
    )?;

    let collateral_value = collection_and_collateral_list.iter().fold(
        Ok(Uint128::zero()),
        |acc, (collection_address, collateral)| -> StdResult<Uint128> {
            let price_item = prices
                .iter()
                .find(|x| x.address == collection_address)
                .ok_or(ContractError::PriceIsNotFound {
                    val: collection_address.to_string(),
                })?;

            if block_time - price_item.price_update_date > common_config.price_update_period {
                Err(ContractError::OutdatedPrice {
                    val: collection_address.to_string(),
                })?;
            }

            let price = helpers::choose_price(&common_config.main_currency, &price_item);

            Ok(acc? + price * Uint128::new(collateral.token_id_list.len() as u128))
        },
    )?;

    if collateral_value.is_zero()
        && !(borrower.accumulated_loan.is_zero() && borrower.loan.amount.is_zero())
    {
        return Ok(Uint128::zero());
    }

    let ltv_max = math::calc_ltv_max(
        rate_config.bid_min_rate,
        rate_config.discount_max_rate,
        rate_config.discount_min_rate,
    );
    let ltv = target_ltv.unwrap_or(ltv_max);
    let accumulated_loan = math::calc_accumulated_loan(
        rate_config.borrow_apr,
        borrower.accumulated_loan,
        borrower.loan.amount,
        block_time,
        borrower.loan.creation_date,
    );
    let loan = borrower.loan.amount + accumulated_loan;

    Ok(math::calc_available_to_borrow(ltv, collateral_value, loan))
}

pub fn query_amounts(deps: Deps, _env: Env) -> StdResult<QueryAmountsResponse> {
    Ok(QueryAmountsResponse {
        unbonders: UNBONDERS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        borrowers: BORROWERS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        collaterals: COLLATERALS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        liquidators: LIQUIDATORS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        liquidation_bids: LIQUIDATION_BIDS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        proposals: PROPOSALS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        collections: COLLECTIONS
            .keys(deps.storage, None, None, Order::Ascending)
            .count() as u32,
    })
}
