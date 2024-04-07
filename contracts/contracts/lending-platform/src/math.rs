use std::cmp::{max, min};

use cosmwasm_std::{Addr, Decimal, Decimal256, StdResult, Uint128};

use goplend_base::{
    converters::{dec256_to_dec, dec256_to_uint128, str_to_dec, u128_to_dec, u128_to_dec256},
    error::ContractError,
    lending_platform::{
        state::{MAX_DEPOSIT_APR, YEAR_IN_SECONDS},
        types::{BiddedCollateralItem, Collateral, LiquidationBid},
    },
    oracle::types::PriceItem,
};

// expected_rewards = borrow_apr * total_borrowed_amount *
// * (block_time - borrowing_vault_creation_date) / year_in_seconds
pub fn calc_expected_rewards(
    block_time: u64,
    borrowing_vault_creation_date: u64,
    borrow_apr: Decimal,
    total_borrowed_amount: Uint128,
) -> Uint128 {
    let borrowing_period = block_time - borrowing_vault_creation_date;

    (borrow_apr
        * u128_to_dec(
            total_borrowed_amount * Uint128::from(borrowing_period)
                / Uint128::from(YEAR_IN_SECONDS),
        ))
    .to_uint_floor()
}

// bgl = m1 * main * bgl_pool / (main_pool + accumulated_rewards)
pub fn calc_minted_bgl_token_amount(
    main_currency_amount: Uint128,
    main_pool: Uint128,
    bgl_supply: Uint128,
    accumulated_rewards: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Uint128 {
    let decimals_multiplier = u128_to_dec256(10u128.pow(bgl_currency_decimals as u32))
        / u128_to_dec256(10u128.pow(main_currency_decimals as u32));

    dec256_to_uint128(if bgl_supply.is_zero() {
        // initially 1 bgl_token == 1 main_token
        decimals_multiplier * u128_to_dec256(main_currency_amount)
    } else {
        decimals_multiplier * u128_to_dec256(main_currency_amount * bgl_supply)
            / u128_to_dec256(main_pool + accumulated_rewards)
    })
}

// main = m2 * bgl * (main_pool + accumulated_rewards) / bgl_pool
pub fn calc_unbonded_main_token_amount(
    bgl_currency_amount: Uint128,
    main_pool: Uint128,
    bgl_supply: Uint128,
    accumulated_rewards: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Uint128 {
    let decimals_multiplier = u128_to_dec256(10u128.pow(main_currency_decimals as u32))
        / u128_to_dec256(10u128.pow(bgl_currency_decimals as u32));

    dec256_to_uint128(
        decimals_multiplier
            * u128_to_dec256(bgl_currency_amount * (main_pool + accumulated_rewards))
            / u128_to_dec256(bgl_supply),
    )
}

pub fn calc_bgl_to_main_price(
    main_pool: Uint128,
    bgl_supply: Uint128,
    accumulated_rewards: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Decimal {
    let price = if bgl_supply.is_zero() {
        Decimal256::one()
    } else {
        u128_to_dec256(main_pool + accumulated_rewards)
            * u128_to_dec256(10u128.pow(bgl_currency_decimals as u32))
            / (u128_to_dec256(bgl_supply)
                * u128_to_dec256(10u128.pow(main_currency_decimals as u32)))
    };

    dec256_to_dec(price)
}

pub fn calc_conditional_deposit_apr(
    borrow_apr: Decimal,
    borrow_fee_rate: Decimal,
    amount_to_deposit: Uint128,
    total_deposited: Uint128,
    total_unbonded: Uint128,
    regular_borrowed: Uint128,
    reserve_borrowed: Uint128,
) -> Decimal {
    if total_deposited.is_zero() {
        return str_to_dec(MAX_DEPOSIT_APR);
    }

    u128_to_dec((borrow_apr - borrow_fee_rate) * (regular_borrowed + reserve_borrowed))
        / u128_to_dec(amount_to_deposit + total_deposited - total_unbonded)
}

#[allow(clippy::too_many_arguments)]
pub fn calc_conditional_ltv(
    borrow_apr: Decimal,
    amount_to_borrow: Uint128,
    accumulated_loan: Uint128,
    loan: Uint128,
    amount_to_deposit: Uint128,
    collateral: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Decimal {
    let collateral = collateral + amount_to_deposit;
    let accumulated_loan = accumulated_loan + amount_to_borrow;

    if collateral.is_zero() {
        return Decimal::zero();
    }

    let borrow_duration = u128_to_dec(Uint128::from(block_time - loan_creation_date));
    let loan = u128_to_dec(accumulated_loan)
        + u128_to_dec(loan)
            * (Decimal::one() + borrow_apr * borrow_duration / u128_to_dec(YEAR_IN_SECONDS));

    loan / u128_to_dec(collateral)
}

pub fn calc_available_to_borrow(
    target_ltv: Decimal,
    collateral: Uint128,
    loan: Uint128,
) -> Uint128 {
    if collateral.is_zero() {
        return Uint128::zero();
    }

    let target_loan = (target_ltv * u128_to_dec(collateral)).to_uint_floor();

    target_loan - min(loan, target_loan)
}

// ltv_max = (loan - sum((1 - discount) * liquidation_value)) / (collateral - sum(liquidation_value))
//
// 0 % discount
// need to liquidate 2 * 650 = 1_300
// liquidator will pay 1_300
// ltv = (1_700 - 1_300) / (1_950 - 1_300) = 0.615
//
// 5 % discount
// need to liquidate 2 * 650 = 1_300
// liquidator will pay 2 * ceil(650 * 0.95) = 2 * 618 = 1_236
// ltv = (1_700 - 1_236) / (1_950 - 1_300) = 0.7138
//
// ltv_max = (loan - (1 - discount) * liquidation_value) / (collateral - liquidation_value)
// ltv_max * collateral - ltv_max * liquidation_value = loan - (1 - discount) * liquidation_value
// (1 - discount - ltv_max) * liquidation_value = loan - ltv_max * collateral
// liquidation_value = (loan - ltv_max * collateral) / (1 - discount - ltv_max)
// liquidation_value = collateral * (ltv - ltv_max) / (1 - discount - ltv_max)
//
// for discount = 0
// liquidation_value = collateral * (ltv - ltv_max) / (1 - ltv_max)
pub fn calc_liquidation_value(
    collateral: Uint128,
    discount_rate: Decimal,
    ltv: Decimal,
    ltv_max: Decimal,
) -> Uint128 {
    min(
        (u128_to_dec(collateral) * (ltv - ltv_max) / (Decimal::one() - discount_rate - ltv_max))
            .to_uint_ceil(),
        collateral,
    )
}

pub fn calc_discounted_price(price: Uint128, discount: Decimal) -> Uint128 {
    (u128_to_dec(price) * (Decimal::one() - discount)).to_uint_ceil()
}

pub fn calc_accumulated_loan(
    borrow_apr: Decimal,
    accumulated_loan: Uint128,
    loan: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Uint128 {
    (u128_to_dec(accumulated_loan)
        + borrow_apr * u128_to_dec(loan * Uint128::from(block_time - loan_creation_date))
            / u128_to_dec(YEAR_IN_SECONDS))
    .to_uint_floor()
}

pub fn calc_bid_min_multiplier(bid_min_rate: Decimal) -> Decimal {
    Decimal::one() + bid_min_rate
}

fn calc_bid_max_multiplier(
    bid_min_rate: Decimal,
    discount_max_rate: Decimal,
    discount_min_rate: Decimal,
) -> Decimal {
    let one = Decimal::one();

    calc_bid_min_multiplier(bid_min_rate) * (one - discount_min_rate) / (one - discount_max_rate)
}

pub fn calc_ltv_max(
    bid_min_rate: Decimal,
    discount_max_rate: Decimal,
    discount_min_rate: Decimal,
) -> Decimal {
    Decimal::one() / calc_bid_max_multiplier(bid_min_rate, discount_max_rate, discount_min_rate)
}

pub fn calc_platform_revenue_on_liquidation(
    liquidation_value: Uint128,
    liquidation_fee_rate: Decimal,
) -> Uint128 {
    (u128_to_dec(liquidation_value) * liquidation_fee_rate).to_uint_ceil()
}

pub fn calc_platform_revenue_on_repay(
    repay_amount: Uint128,
    borrow_apr: Decimal,
    borrow_fee_rate: Decimal,
) -> Uint128 {
    (u128_to_dec(repay_amount) * borrow_fee_rate / borrow_apr).to_uint_ceil()
}

pub fn calc_min_available_liquidity(
    reserve_deposited: Uint128,
    min_available_liquidity_multiplier: Decimal,
) -> Uint128 {
    (min_available_liquidity_multiplier * u128_to_dec(reserve_deposited)).to_uint_ceil()
}

pub fn calc_available_to_borrow_regular_liquidity(
    regular_deposited: Uint128,
    accumulated_rewards: Uint128,
    unbonded: Uint128,
    regular_borrowed: Uint128,
) -> Uint128 {
    max(
        regular_deposited + accumulated_rewards - unbonded,
        regular_borrowed,
    ) - regular_borrowed
}

pub fn calc_available_to_borrow_reserve_liquidity(
    reserve_deposited: Uint128,
    min_available_liquidity: Uint128,
    reserve_borrowed: Uint128,
) -> Uint128 {
    max(
        reserve_deposited - min_available_liquidity,
        reserve_borrowed,
    ) - reserve_borrowed
}

pub fn calc_total_available_to_borrow_liquidity(
    regular_deposited: Uint128,
    accumulated_rewards: Uint128,
    unbonded: Uint128,
    regular_borrowed: Uint128,
    reserve_deposited: Uint128,
    min_available_liquidity: Uint128,
    reserve_borrowed: Uint128,
) -> Uint128 {
    calc_available_to_borrow_regular_liquidity(
        regular_deposited,
        accumulated_rewards,
        unbonded,
        regular_borrowed,
    ) + calc_available_to_borrow_reserve_liquidity(
        reserve_deposited,
        min_available_liquidity,
        reserve_borrowed,
    )
}

/// places bid on each collateral token, best bids have higher priority
pub fn match_collaterals_and_bids(
    collection_and_collateral_list_by_borrower: &[(Addr, Collateral)],
    collection_and_bid_list: &[(Addr, LiquidationBid)],
    price_list: &[PriceItem],
) -> StdResult<Vec<BiddedCollateralItem>> {
    let mut collection_and_bid_list = collection_and_bid_list.to_owned();

    // set bids priority
    collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.amount);
    collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.creation_date);
    collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.discount);
    collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.is_offer);

    // add best bid for each collateral token
    let mut bidded_collateral_list: Vec<BiddedCollateralItem> = vec![];

    for (collateral_collection, collateral) in collection_and_collateral_list_by_borrower {
        let PriceItem { price, .. } = price_list
            .iter()
            .find(|x| x.address == collateral_collection)
            .ok_or(ContractError::PriceIsNotFound {
                val: collateral_collection.to_string(),
            })?;

        for token_id in collateral.token_id_list.clone() {
            for (_bid_collection, bid) in collection_and_bid_list.iter_mut() {
                let discounted_price = calc_discounted_price(price.amount, bid.discount);

                if bid.token_id_list.contains(&token_id) && bid.amount >= discounted_price {
                    bidded_collateral_list.push(BiddedCollateralItem {
                        collection: collateral_collection.clone(),
                        token_id: token_id.clone(),
                        owner: collateral.owner.clone(),
                        liquidator: bid.liquidator.clone(),
                        bid_creation_date: bid.creation_date,
                        liquidation_price: discounted_price,
                        collateral_price: price.amount,
                        bid_discount: bid.discount,
                        is_offer: bid.is_offer,
                    });

                    bid.amount -= discounted_price;
                    break;
                }
            }
        }
    }

    Ok(bidded_collateral_list)
}

fn calc_liquidation_ltv(
    bidded_collateral: &[BiddedCollateralItem],
    borrow_apr: Decimal,
    accumulated_loan: Uint128,
    loan: Uint128,
    collateral: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Decimal {
    let loan_diff = bidded_collateral
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur.liquidation_price);
    let collateral_diff = bidded_collateral
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur.collateral_price);

    let collateral = collateral - collateral_diff;

    if collateral.is_zero() {
        return Decimal::zero();
    }

    let borrow_duration = u128_to_dec(Uint128::from(block_time - loan_creation_date));
    let loan = u128_to_dec(accumulated_loan)
        + u128_to_dec(loan)
            * (Decimal::one() + borrow_apr * borrow_duration / u128_to_dec(YEAR_IN_SECONDS))
        - u128_to_dec(loan_diff);

    loan / u128_to_dec(collateral)
}

// there is no way to calculate liquidation_value properly
// then ltv must be calculated on each iteration
#[allow(clippy::too_many_arguments)]
pub fn calc_liquidation_set(
    bidded_collateral_list: &[BiddedCollateralItem],
    max_liquidation_value: Uint128,
    ltv_max: Decimal,
    borrow_apr: Decimal,
    accumulated_loan: Uint128,
    loan: Uint128,
    collateral: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Vec<BiddedCollateralItem> {
    const UPPER_VALUE_MULTIPLIER: &str = "1.5";

    let mut bidded_collateral_list = bidded_collateral_list.to_owned();

    // sort ascending by creation_date
    bidded_collateral_list.sort_unstable_by_key(|x| x.bid_creation_date);
    // sort ascending by discount
    bidded_collateral_list.sort_unstable_by_key(|x| x.bid_discount);
    // sort descending by liquidation_price
    bidded_collateral_list.sort_unstable_by_key(|x| Uint128::MAX - x.liquidation_price);

    // get subvector with values <= liquidation_value
    // [19, 16, 14, 13, 10, 7, 5, 5, 2] -> [10, 7, 5, 5, 2]
    let subvector: Vec<BiddedCollateralItem> = bidded_collateral_list
        .iter()
        .cloned()
        .filter(|x| x.liquidation_price <= max_liquidation_value)
        .collect();

    // get smallest collateral_list item > liquidation_value or 1.5 * liquidation_value if it isn't found
    let upper_value = bidded_collateral_list
        .iter()
        .rev()
        .find(|x| x.liquidation_price > max_liquidation_value)
        .map(|x| x.liquidation_price)
        .unwrap_or(
            (u128_to_dec(max_liquidation_value) * str_to_dec(UPPER_VALUE_MULTIPLIER))
                .to_uint_floor(),
        )
        .to_owned();

    // iterate over subvector, compare with [liquidation_value, upper_value] and generate vectors
    let mut result: Vec<Vec<BiddedCollateralItem>> = vec![];

    for (i, bidded_collateral_item) in subvector.iter().cloned().enumerate() {
        let mut temp: Vec<BiddedCollateralItem> = vec![bidded_collateral_item.clone()];
        let mut collateral_sum: Uint128 = bidded_collateral_item.liquidation_price;

        let ltv = calc_liquidation_ltv(
            &temp,
            borrow_apr,
            accumulated_loan,
            loan,
            collateral,
            block_time,
            loan_creation_date,
        );

        if ltv <= ltv_max {
            result.push(temp.clone());
            continue;
        }

        for bidded_collateral_item_next in &subvector[(i + 1)..subvector.len()] {
            let collateral_sum_next =
                collateral_sum + bidded_collateral_item_next.liquidation_price.to_owned();

            if collateral_sum_next >= upper_value {
                continue;
            }

            temp.push(bidded_collateral_item_next.to_owned());

            let ltv = calc_liquidation_ltv(
                &temp,
                borrow_apr,
                accumulated_loan,
                loan,
                collateral,
                block_time,
                loan_creation_date,
            );

            if ltv <= ltv_max {
                result.push(temp.clone());
                temp.pop();
                continue;
            }

            collateral_sum = collateral_sum_next;
        }
    }

    // liquidate first more expensive collateral items
    result.sort_unstable_by_key(|x| x.len());
    // prefer sets with lower sum discount
    result.sort_unstable_by_key(|x| {
        x.iter()
            .fold(Decimal::zero(), |acc, cur| acc + cur.bid_discount)
    });
    // choose cheapest set
    result.sort_unstable_by_key(|x| {
        x.iter()
            .fold(Uint128::zero(), |acc, cur| acc + cur.liquidation_price)
    });

    result
        .first()
        .map(|x| x.to_owned())
        .unwrap_or(bidded_collateral_list.to_owned())
}
