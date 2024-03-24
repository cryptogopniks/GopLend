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

pub fn calc_conditional_deposit_apr(
    borrow_apr: Decimal,
    borrow_fee_rate: Decimal,
    amount_to_deposit: Uint128,
    total_deposited: Uint128,
    total_unbonded: Uint128,
    total_borrowed: Uint128,
) -> Decimal {
    if total_deposited.is_zero() {
        return str_to_dec(MAX_DEPOSIT_APR);
    }

    u128_to_dec((borrow_apr - borrow_fee_rate) * total_borrowed)
        / u128_to_dec(amount_to_deposit + total_deposited - total_unbonded)
}

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

    if collateral.is_zero() && accumulated_loan.is_zero() && loan.is_zero() {
        return Decimal::zero();
    }

    let borrow_duration = u128_to_dec(Uint128::from(block_time - loan_creation_date));
    let loan = u128_to_dec(accumulated_loan)
        + u128_to_dec(loan)
            * (Decimal::one() + borrow_apr * borrow_duration / u128_to_dec(YEAR_IN_SECONDS));

    loan / u128_to_dec(collateral)
}

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
pub fn calc_liquidation_value(collateral: Uint128, ltv: Decimal, ltv_max: Decimal) -> Uint128 {
    min(
        (u128_to_dec(collateral) * (ltv - ltv_max) / (Decimal::one() - ltv_max)).to_uint_ceil(),
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

/// places bid on each collateral token, best bids have higher priority
pub fn match_collaterals_and_bids(
    collection_and_collateral_list_by_borrower: &Vec<(Addr, Collateral)>,
    collection_and_bid_list: &Vec<(Addr, LiquidationBid)>,
    price_list: &Vec<PriceItem>,
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
            for (bid_collection, bid) in collection_and_bid_list.iter_mut() {
                let discounted_price = calc_discounted_price(price.amount, bid.discount);

                if bid.token_id_list.contains(&token_id) && bid.amount >= discounted_price {
                    bidded_collateral_list.push(BiddedCollateralItem {
                        collection: bid_collection.clone(),
                        token_id: token_id.clone(),
                        owner: collateral.owner.clone(),
                        liquidator: bid.liquidator.clone(),
                        bid_creation_date: bid.creation_date,
                        liquidation_price: discounted_price,
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

pub fn calc_liquidation_sets(
    bidded_collateral_list: &Vec<BiddedCollateralItem>,
    liquidation_value: Uint128,
) -> Vec<Vec<BiddedCollateralItem>> {
    let mut bidded_collateral_list = bidded_collateral_list.clone();

    // sort ascending by creation_date
    bidded_collateral_list.sort_unstable_by_key(|x| x.bid_creation_date);
    // sort ascending by discount
    bidded_collateral_list.sort_unstable_by_key(|x| x.bid_discount);
    // sort descending by liquidation_price
    bidded_collateral_list.sort_unstable_by_key(|x| Uint128::MAX - x.liquidation_price);

    // println!("{:#?}", bidded_collateral_list);

    // get subvector with values <= liquidation_value
    // [19, 16, 14, 13, 10, 7, 5, 5, 2] -> [10, 7, 5, 5, 2]
    let subvector: Vec<BiddedCollateralItem> = bidded_collateral_list
        .iter()
        .cloned()
        .filter(|x| x.liquidation_price <= liquidation_value)
        .collect();

    // get smallest collateral_list item > liquidation_value or liquidation_value if it isn't found
    let upper_value = bidded_collateral_list
        .iter()
        .rev()
        .find(|x| x.liquidation_price > liquidation_value)
        .map(|x| x.liquidation_price)
        .unwrap_or(liquidation_value)
        .to_owned();

    // println!("{:#?}", (upper_value, &subvector));

    // iterate over subvector, compare with [liquidation_value, upper_value] and generate vectors
    let mut result: Vec<Vec<BiddedCollateralItem>> = vec![];

    for (i, bidded_collateral_item) in subvector.iter().cloned().enumerate() {
        let mut temp: Vec<BiddedCollateralItem> = vec![bidded_collateral_item.clone()];
        let mut collateral_sum: Uint128 = bidded_collateral_item.liquidation_price;

        if collateral_sum >= liquidation_value && collateral_sum < upper_value {
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

            if collateral_sum_next >= liquidation_value && collateral_sum_next < upper_value {
                result.push(temp.clone());
                temp.pop();
                continue;
            }

            collateral_sum = collateral_sum_next;
        }
    }

    println!("{:#?}", result);

    result
}
