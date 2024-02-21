use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, StdResult, Uint128};

use goplend_base::{
    assets::{Funds, Token},
    error::ContractError,
    lending_platform::{
        msg::{
            QueryBorrowersResponseItem, QueryCollateralsResponseItem, QueryCollectionsResponseItem,
            QueryDepositorsResponseItem, QueryLiquidationBidsResponseItem,
            QueryLiquidatorsResponseItem, QueryProposalsResponseItem,
        },
        state::{
            BALANCES, BLOCK_COUNTER, BORROWERS, COLLATERALS, COLLECTIONS, CONFIG, DEPOSITORS,
            LIQUIDATION_BIDS, LIQUIDATORS, PLATFORM_REVENUE, PROPOSALS,
        },
        types::{
            Balances, Borrower, Collateral, Collection, Config, Depositor, LiquidationBid,
            Liquidator,
        },
    },
    oracle::{self, state::PRICE_UPDATE_PERIOD},
    utils::{filter_by_address_list, unwrap_field},
};

use crate::math::{calc_bgl_to_main_price, calc_deposit_apr, calc_ltv};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_platform_revenue(deps: Deps, _env: Env) -> StdResult<Vec<Funds<Token>>> {
    PLATFORM_REVENUE.load(deps.storage)
}

pub fn query_balances(deps: Deps, _env: Env) -> StdResult<Balances> {
    BALANCES.load(deps.storage)
}

pub fn query_depositors(
    deps: Deps,
    _env: Env,
    addresses: Option<Vec<String>>,
) -> StdResult<Vec<QueryDepositorsResponseItem>> {
    let depositors: Vec<(Addr, Depositor)> = DEPOSITORS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    Ok(filter_by_address_list(deps.api, &addresses, &depositors)?
        .into_iter()
        .map(|(address, depositor)| QueryDepositorsResponseItem { address, depositor })
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
    let collaterals: Vec<(Addr, Collateral)> = COLLATERALS
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

// TODO: linear bgl growth (considering borrow_apr)
pub fn query_bgl_currency_to_main_currency_price(deps: Deps, _env: Env) -> StdResult<Decimal> {
    let balances = BALANCES.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let main_currency = config.main_currency;
    let bgl_currency = unwrap_field(config.bgl_currency, "bgl_currency")?;

    Ok(calc_bgl_to_main_price(
        balances.total_main_currency_pool,
        balances.total_bgl_currency_supply,
        main_currency.decimals,
        bgl_currency.decimals,
    ))
}

pub fn query_deposit_apr(deps: Deps, _env: Env) -> StdResult<Decimal> {
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

    Ok(calc_deposit_apr(
        borrow_apr,
        borrow_fee_rate,
        total_deposited,
        total_unbonded,
        total_borrowed,
    ))
}

// TODO: optimize
pub fn query_ltv(deps: Deps, env: Env, borrower: String) -> StdResult<Decimal> {
    let block_time = env.block.time.seconds();
    let Config {
        borrow_apr, oracle, ..
    } = CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(oracle, "oracle")?;
    let borrower_address = deps.api.addr_validate(&borrower)?;
    let borrower = BORROWERS.load(deps.storage, &borrower_address)?;

    // get collaterals owned by borrower
    let collection_and_collateral_list: Vec<(Addr, Collateral)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .filter(|(_collection, collateral)| collateral.owner == &borrower_address)
        .collect();

    let collection_addresses: Vec<String> = collection_and_collateral_list
        .iter()
        .map(|(collection, _collateral)| collection.to_string())
        .collect();

    let prices: Vec<oracle::msg::QueryPricesResponseItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
        },
    )?;

    let collateral_value = collection_and_collateral_list.iter().fold(
        Ok(Uint128::zero()),
        |acc, (collection, _collateral)| -> StdResult<Uint128> {
            let oracle::msg::QueryPricesResponseItem {
                price,
                price_update_date,
                ..
            } = prices.iter().find(|x| x.address == collection).ok_or(
                ContractError::PriceIsNotFound {
                    val: collection.to_string(),
                },
            )?;

            if block_time - price_update_date > PRICE_UPDATE_PERIOD {
                Err(ContractError::OutdatedPrice {
                    val: collection.to_string(),
                })?;
            }

            let (_collection, collateral) = collection_and_collateral_list
                .iter()
                .find(|(collection_address, ..)| collection_address == collection)
                .ok_or(ContractError::Unauthorized)?;

            // TODO: decimals, other currencies
            Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
        },
    )?;

    Ok(calc_ltv(
        borrow_apr,
        borrower.accumulated_loan,
        borrower.loan.amount,
        collateral_value,
        block_time,
        borrower.loan.creation_date,
    ))
}

pub fn query_block_counter(deps: Deps, _env: Env) -> StdResult<u64> {
    Ok(BLOCK_COUNTER.load(deps.storage).unwrap_or_default())
}
