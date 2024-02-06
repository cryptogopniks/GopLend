use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, StdResult, Uint128};

use goplend_base::{
    assets::{Funds, Token},
    converters::u128_to_dec,
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
    utils::{filter_by_address_list, unwrap_field},
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

pub fn query_bgl_currency_to_main_currency_price(deps: Deps, _env: Env) -> StdResult<Decimal> {
    let balances = BALANCES.load(deps.storage)?;
    // TODO: add decimals
    // let config = CONFIG.load(deps.storage)?;
    // let bgl_currency = unwrap_field(config.bgl_currency, "bgl_currency")?;

    let price = if balances.total_bgl_currency_supply.is_zero() {
        Decimal::one()
    } else {
        u128_to_dec(balances.total_main_currency_pool)
            / u128_to_dec(balances.total_bgl_currency_supply)
    };

    Ok(price)
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

    let deposit_apr = u128_to_dec((borrow_apr - borrow_fee_rate) * total_borrowed)
        / u128_to_dec(std::cmp::max(total_deposited, Uint128::one()) - total_unbonded);

    Ok(deposit_apr)
}

pub fn query_block_counter(deps: Deps, _env: Env) -> StdResult<u64> {
    Ok(BLOCK_COUNTER.load(deps.storage).unwrap_or_default())
}
