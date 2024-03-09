use std::cmp::{max, min};

use cosmwasm_std::{Decimal, Decimal256, Uint128};

use goplend_base::{
    converters::{dec256_to_dec, dec256_to_uint128, str_to_dec, u128_to_dec, u128_to_dec256},
    lending_platform::state::{MAX_DEPOSIT_APR, YEAR_IN_SECONDS},
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

// bgl = m1 * main * bgl_pool / (main_pool + accumulated_expected_rewards)
pub fn calc_minted_bgl_token_amount(
    main_currency_amount: Uint128,
    total_main_currency_pool: Uint128,
    total_bgl_currency_supply: Uint128,
    accumulated_expected_rewards: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Uint128 {
    let decimals_multiplier = u128_to_dec256(10u128.pow(bgl_currency_decimals as u32))
        / u128_to_dec256(10u128.pow(main_currency_decimals as u32));

    dec256_to_uint128(if total_bgl_currency_supply.is_zero() {
        // initially 1 bgl_token == 1 main_token
        decimals_multiplier * u128_to_dec256(main_currency_amount)
    } else {
        decimals_multiplier * u128_to_dec256(main_currency_amount * total_bgl_currency_supply)
            / u128_to_dec256(total_main_currency_pool + accumulated_expected_rewards)
    })
}

// main = m2 * bgl * (main_pool + accumulated_expected_rewards) / bgl_pool
pub fn calc_unbonded_main_token_amount(
    bgl_currency_amount: Uint128,
    total_main_currency_pool: Uint128,
    total_bgl_currency_supply: Uint128,
    accumulated_expected_rewards: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Uint128 {
    let decimals_multiplier = u128_to_dec256(10u128.pow(main_currency_decimals as u32))
        / u128_to_dec256(10u128.pow(bgl_currency_decimals as u32));

    dec256_to_uint128(
        decimals_multiplier
            * u128_to_dec256(
                bgl_currency_amount * (total_main_currency_pool + accumulated_expected_rewards),
            )
            / u128_to_dec256(total_bgl_currency_supply),
    )
}

pub fn calc_bgl_to_main_price(
    total_main_currency_pool: Uint128,
    total_bgl_currency_supply: Uint128,
    accumulated_expected_rewards: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Decimal {
    let price = if total_bgl_currency_supply.is_zero() {
        Decimal256::one()
    } else {
        u128_to_dec256(total_main_currency_pool + accumulated_expected_rewards)
            * u128_to_dec256(10u128.pow(bgl_currency_decimals as u32))
            / (u128_to_dec256(total_bgl_currency_supply)
                * u128_to_dec256(10u128.pow(main_currency_decimals as u32)))
    };

    dec256_to_dec(price)
}

pub fn calc_deposit_apr(
    borrow_apr: Decimal,
    borrow_fee_rate: Decimal,
    total_deposited: Uint128,
    total_unbonded: Uint128,
    total_borrowed: Uint128,
) -> Decimal {
    if total_deposited.is_zero() {
        return str_to_dec(MAX_DEPOSIT_APR);
    }

    u128_to_dec((borrow_apr - borrow_fee_rate) * total_borrowed)
        / u128_to_dec(total_deposited - total_unbonded)
}

pub fn calc_ltv(
    borrow_apr: Decimal,
    accumulated_loan: Uint128,
    loan: Uint128,
    collateral: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Decimal {
    if collateral.is_zero() && accumulated_loan.is_zero() && loan.is_zero() {
        return Decimal::zero();
    }

    let borrow_duration = u128_to_dec(Uint128::from(block_time - loan_creation_date));
    let loan = u128_to_dec(accumulated_loan)
        + u128_to_dec(loan)
            * (Decimal::one() + borrow_apr * borrow_duration / u128_to_dec(YEAR_IN_SECONDS));

    loan / u128_to_dec(collateral)
}

// liquidation_value = collateral * (ltv - ltv_max) / (1 - ltv_max)
pub fn calc_liquidation_value(collateral: Uint128, ltv: Decimal, ltv_max: Decimal) -> Uint128 {
    min(
        (u128_to_dec(collateral) * (ltv - ltv_max) / (Decimal::one() - ltv_max)).to_uint_ceil(),
        collateral,
    )
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
    (u128_to_dec(repay_amount) * borrow_fee_rate / borrow_apr).to_uint_floor()
}

pub fn calc_min_available_liquidity(
    min_available_liquidity_multiplier: Decimal,
    total_main_currency_pool: Uint128,
    reserve_main_currency_pool: Uint128,
    accumulated_expected_rewards: Uint128,
) -> Uint128 {
    (min_available_liquidity_multiplier
        * u128_to_dec(
            total_main_currency_pool + accumulated_expected_rewards - reserve_main_currency_pool,
        ))
    .to_uint_ceil()
}

pub fn calc_total_available_to_borrow_liquidity(
    total_main_currency_pool: Uint128,
    total_borrowed_amount: Uint128,
    min_available_liquidity: Uint128,
) -> Uint128 {
    max(
        total_main_currency_pool - total_borrowed_amount,
        min_available_liquidity,
    ) - min_available_liquidity
}
