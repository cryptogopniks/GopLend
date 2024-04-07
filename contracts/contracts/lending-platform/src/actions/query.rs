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
            PLATFORM_REVENUE, PROPOSALS, RATE_CONFIG, UNBONDERS,
        },
        types::{
            AddressConfig, Balances, Borrower, Collateral, Collection, CommonConfig,
            LiquidationBid, Liquidator, RateConfig, Unbonder,
        },
    },
    oracle,
    utils::unwrap_field,
};

use crate::math;

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

pub fn query_balances(deps: Deps, _env: Env) -> StdResult<Balances> {
    BALANCES.load(deps.storage)
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
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<QueryBorrowersResponseItem>> {
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
            let (address, borrower) = x.unwrap();
            QueryBorrowersResponseItem { address, borrower }
        })
        .collect())
}

pub fn query_borrower(deps: Deps, _env: Env, address: String) -> StdResult<Borrower> {
    Ok(BORROWERS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default())
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
    let common_config = COMMON_CONFIG.load(deps.storage)?;
    let main_currency = common_config.main_currency;
    let bgl_currency = unwrap_field(common_config.bgl_currency, "bgl_currency")?;

    let expected_rewards = math::calc_expected_rewards(
        block_time,
        balances.regular.borrowed.creation_date,
        rate_config.borrow_apr,
        balances.regular.borrowed.amount,
    );
    let rewards = balances.regular.accumulated_rewards + expected_rewards;

    Ok(math::calc_bgl_to_main_price(
        balances.regular.main_pool,
        balances.regular.bgl_supply,
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

    let all_collaterals: Vec<(Addr, Vec<Collateral>)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    BORROWERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .flatten()
        .map(|(borrower_address, borrower)| {
            // get collaterals owned by borrower
            let collection_and_collateral_list: Vec<(Addr, Collateral)> = all_collaterals
                .iter()
                .filter(|(_collection, collaterals)| {
                    collaterals
                        .iter()
                        .any(|collateral| collateral.owner == borrower_address)
                })
                .map(
                    |(collection, collaterals)| -> StdResult<(Addr, Collateral)> {
                        // single collateral per collection
                        let collateral = collaterals
                            .iter()
                            .find(|collateral| collateral.owner == borrower_address)
                            .ok_or(ContractError::CollateralIsNotFound)?;

                        Ok((collection.to_owned(), collateral.to_owned()))
                    },
                )
                .collect::<StdResult<Vec<(Addr, Collateral)>>>()?;

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

                    if block_time - price_update_date > common_config.price_update_period {
                        Err(ContractError::OutdatedPrice {
                            val: collection_address.to_string(),
                        })?;
                    }

                    Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
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
    let collection_and_collateral_list: Vec<(Addr, Collateral)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .filter(|(_collection, collaterals)| {
            collaterals
                .iter()
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
            start_after: None,
            limit: None,
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

            if block_time - price_update_date > common_config.price_update_period {
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

    let expected_rewards = math::calc_expected_rewards(
        block_time,
        balances.regular.borrowed.creation_date,
        rate_config.borrow_apr,
        balances.regular.borrowed.amount,
    );
    let rewards = balances.regular.accumulated_rewards + expected_rewards;

    let min_available_liquidity = math::calc_min_available_liquidity(
        balances.reserve.deposited,
        common_config.min_available_liquidity_multiplier,
    );

    let total_available_to_borrow_liquidity = math::calc_total_available_to_borrow_liquidity(
        balances.regular.deposited,
        rewards,
        balances.regular.unbonded,
        balances.regular.borrowed.amount,
        balances.reserve.deposited,
        min_available_liquidity,
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
    let collection_and_collateral_list: Vec<(Addr, Collateral)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .filter(|(_collection, collaterals)| {
            collaterals
                .iter()
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
            start_after: None,
            limit: None,
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

            if block_time - price_update_date > common_config.price_update_period {
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
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        borrowers: BORROWERS
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        collaterals: COLLATERALS
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        liquidators: LIQUIDATORS
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        liquidation_bids: LIQUIDATION_BIDS
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        proposals: PROPOSALS
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
        collections: COLLECTIONS
            .range(deps.storage, None, None, Order::Ascending)
            .count() as u32,
    })
}

// pub fn query_targets(
//     deps: Deps,
//     env: Env,
//     borrowers: Vec<String>,
// ) -> StdResult<Vec<LiquidationItem>> {
//     let block_time = env.block.time.seconds();
//     let address_config = ADDRESS_CONFIG.load(deps.storage)?;
//     let rate_config = RATE_CONFIG.load(deps.storage)?;
//     let common_config = COMMON_CONFIG.load(deps.storage)?;
//     let oracle = &unwrap_field(address_config.oracle, "oracle")?;

//     let borrowers = get_borrowers(deps.storage, deps.api, &borrowers)?;

//     // get all collaterals
//     let collection_and_collateral_list: Vec<(Addr, Vec<Collateral>)> = COLLATERALS
//         .range(deps.storage, None, None, Order::Ascending)
//         .flatten()
//         .collect();

//     // get collection prices
//     let prices = get_collection_prices(deps, &borrowers, &collection_and_collateral_list, oracle)?;

//     // accumulators
//     let mut targets: Vec<LiquidationItem> = vec![];

//     // iterate over borrowers
//     for (
//         borrower_address,
//         Borrower {
//             accumulated_loan,
//             loan,
//         },
//     ) in borrowers
//     {
//         // get collaterals owned by borrower
//         let collection_and_collateral_list_by_borrower =
//             get_collaterals_by_borrower(&collection_and_collateral_list, &borrower_address)?;

//         let collateral_value_option = get_collateral_value(
//             &collection_and_collateral_list_by_borrower,
//             &prices,
//             block_time,
//             common_config.price_update_period,
//         )?;
//         // go to next borrower if any price is outdated
//         if collateral_value_option.is_none() {
//             continue;
//         }
//         let collateral_value = collateral_value_option.unwrap();

//         // check ltv
//         let ltv_max = math::calc_ltv_max(
//             rate_config.bid_min_rate,
//             rate_config.discount_max_rate,
//             rate_config.discount_min_rate,
//         );
//         let ltv = math::calc_conditional_ltv(
//             rate_config.borrow_apr,
//             Uint128::zero(),
//             accumulated_loan,
//             loan.amount,
//             Uint128::zero(),
//             collateral_value,
//             block_time,
//             loan.creation_date,
//         );

//         // if ltv is low check next borrower, otherwise liquidate required amount
//         if ltv <= ltv_max {
//             continue;
//         }

//         // find bids containing at least 1 borrower's collateral token
//         let collection_and_bid_list =
//             get_collection_and_bid_list(deps.storage, &collection_and_collateral_list_by_borrower)?;

//         let bidded_collateral_list = math::match_collaterals_and_bids(
//             &collection_and_collateral_list_by_borrower,
//             &collection_and_bid_list,
//             &prices,
//         )?;

//         // calculate how much tokens and what collection must be liquidated to get proper ltv
//         // for worst case (max discount)
//         let max_liquidation_value = math::calc_liquidation_value(
//             collateral_value,
//             rate_config.discount_max_rate,
//             ltv,
//             ltv_max,
//         );

//         let liquidation_set = math::calc_liquidation_set(
//             &bidded_collateral_list,
//             max_liquidation_value,
//             ltv_max,
//             rate_config.borrow_apr,
//             accumulated_loan,
//             loan.amount,
//             collateral_value,
//             block_time,
//             loan.creation_date,
//         );

//         targets.push(LiquidationItem {
//             borrower: borrower_address.to_string(),
//             liquidation_set,
//         });
//     }

//     Ok(targets)
// }
