use cosmwasm_std::{Decimal, Decimal256, Uint128};

use goplend_base::converters::{dec256_to_dec, dec256_to_uint128, u128_to_dec, u128_to_dec256};

// bgl = m1 * main * bgl_pool / main_pool
pub fn calc_minted_bgl_token_amount(
    main_currency_amount: Uint128,
    total_main_currency_pool: Uint128,
    total_bgl_currency_supply: Uint128,
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
            / u128_to_dec256(total_main_currency_pool)
    })
}

// main = m2 * bgl * main_pool / bgl_pool
pub fn calc_unbonded_main_token_amount(
    bgl_currency_amount: Uint128,
    total_main_currency_pool: Uint128,
    total_bgl_currency_supply: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Uint128 {
    let decimals_multiplier = u128_to_dec256(10u128.pow(main_currency_decimals as u32))
        / u128_to_dec256(10u128.pow(bgl_currency_decimals as u32));

    dec256_to_uint128(
        decimals_multiplier * u128_to_dec256(bgl_currency_amount * total_main_currency_pool)
            / u128_to_dec256(total_bgl_currency_supply),
    )
}

pub fn calc_bgl_to_main_price(
    total_main_currency_pool: Uint128,
    total_bgl_currency_supply: Uint128,
    main_currency_decimals: u8,
    bgl_currency_decimals: u8,
) -> Decimal {
    let price = if total_bgl_currency_supply.is_zero() {
        Decimal256::one()
    } else {
        u128_to_dec256(total_main_currency_pool)
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
    u128_to_dec((borrow_apr - borrow_fee_rate) * total_borrowed)
        / u128_to_dec(std::cmp::max(total_deposited, Uint128::one()) - total_unbonded)
}

pub fn calc_ltv(
    borrow_apr: Decimal,
    accumulated_loan: Uint128,
    loan: Uint128,
    collateral: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Decimal {
    let loan = u128_to_dec(accumulated_loan)
        + borrow_apr * u128_to_dec(loan * Uint128::from(block_time - loan_creation_date));

    loan / u128_to_dec(collateral)
}

pub fn calc_accumulated_loan(
    borrow_apr: Decimal,
    accumulated_loan: Uint128,
    loan: Uint128,
    block_time: u64,
    loan_creation_date: u64,
) -> Uint128 {
    (u128_to_dec(accumulated_loan)
        + borrow_apr * u128_to_dec(loan * Uint128::from(block_time - loan_creation_date)))
    .to_uint_floor()
}

pub fn calc_bid_min_multiplier(bid_min_rate: Decimal) -> Decimal {
    Decimal::one() + bid_min_rate
}

pub fn calc_bid_max_multiplier(
    bid_min_rate: Decimal,
    discount_max_rate: Decimal,
    discount_min_rate: Decimal,
) -> Decimal {
    calc_bid_min_multiplier(bid_min_rate) / (discount_max_rate - discount_min_rate)
}
