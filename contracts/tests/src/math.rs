use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};
use goplend_base::{
    assets::{Currency, Funds, Token},
    converters::{str_to_dec, u128_to_dec},
    lending_platform::types::{BiddedCollateralItem, Collateral, LiquidationBid},
    oracle::types::PriceItem,
};
use lending_platform::math;
use speculoos::assert_that;

use crate::helpers::suite::types::{
    GetDecimals, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken,
};

fn to_string_vec(str_vec: Vec<&str>) -> Vec<String> {
    str_vec.into_iter().map(|x| x.to_string()).collect()
}

#[test]
fn match_collaterals_and_bids_default() -> StdResult<()> {
    let collection_and_collateral_list_by_borrower: Vec<(Addr, Collateral)> = vec![
        (
            ProjectNft::Gopniks.into(),
            Collateral {
                owner: ProjectAccount::Owner.into(),
                token_id_list: to_string_vec(vec!["1", "7", "4"]),
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            Collateral {
                owner: ProjectAccount::Owner.into(),
                token_id_list: to_string_vec(vec!["1", "8", "2", "5"]),
            },
        ),
    ];

    let price_list: Vec<PriceItem> = vec![
        PriceItem {
            address: ProjectNft::Gopniks.into(),
            price: Funds::new(
                Uint128::new(200),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
            price_update_date: 0,
        },
        PriceItem {
            address: ProjectNft::Pinjeons.into(),
            price: Funds::new(
                Uint128::new(100),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
            price_update_date: 1_000,
        },
    ];

    let collection_and_bid_list: Vec<(Addr, LiquidationBid)> = vec![
        // gopnics
        // too small
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 100,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["1"]),
                amount: Uint128::new(75),
                discount: str_to_dec("0"),
                is_offer: false,
            },
        ),
        // out of scope
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 100,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["42"]),
                amount: Uint128::new(750),
                discount: str_to_dec("0"),
                is_offer: false,
            },
        ),
        // 2 tokens bid, 1 token funds
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["1", "4"]),
                amount: Uint128::new(300),
                discount: str_to_dec("0.1"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["1", "4"]),
                amount: Uint128::new(300),
                discount: str_to_dec("0.05"),
                is_offer: false,
            },
        ),
        // offer
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["7"]),
                amount: Uint128::new(900),
                discount: str_to_dec("0"),
                is_offer: true,
            },
        ),
        // pingeons
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["2"]),
                amount: Uint128::new(150),
                discount: str_to_dec("0.1"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["2"]),
                amount: Uint128::new(125),
                discount: str_to_dec("0.1"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["5"]),
                amount: Uint128::new(125),
                discount: str_to_dec("0"),
                is_offer: true,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["1", "8", "42"]),
                amount: Uint128::new(125),
                discount: str_to_dec("0.05"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["8", "42"]),
                amount: Uint128::new(225),
                discount: str_to_dec("0.2"),
                is_offer: false,
            },
        ),
    ];

    let collaterals_and_bids = math::match_collaterals_and_bids(
        &collection_and_collateral_list_by_borrower,
        &collection_and_bid_list,
        &price_list,
    )?;

    assert_that(&collaterals_and_bids).is_equal_to(vec![
        BiddedCollateralItem {
            collection: ProjectNft::Gopniks.into(),
            token_id: "1".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Bob.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(190),
            collateral_price: Uint128::new(200),
            bid_discount: str_to_dec("0.05"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Gopniks.into(),
            token_id: "7".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Bob.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(200),
            collateral_price: Uint128::new(200),
            bid_discount: str_to_dec("0"),
            is_offer: true,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Gopniks.into(),
            token_id: "4".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(180),
            collateral_price: Uint128::new(200),
            bid_discount: str_to_dec("0.1"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Pinjeons.into(),
            token_id: "1".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Bob.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(95),
            collateral_price: Uint128::new(100),
            bid_discount: str_to_dec("0.05"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Pinjeons.into(),
            token_id: "8".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(80),
            collateral_price: Uint128::new(100),
            bid_discount: str_to_dec("0.2"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Pinjeons.into(),
            token_id: "2".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(90),
            collateral_price: Uint128::new(100),
            bid_discount: str_to_dec("0.1"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Pinjeons.into(),
            token_id: "5".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(100),
            collateral_price: Uint128::new(100),
            bid_discount: str_to_dec("0"),
            is_offer: true,
        },
    ]);

    Ok(())
}

#[test]
fn calc_liquidation_sets_default() {
    let collateral_list: Vec<BiddedCollateralItem> = vec![
        (5, 10),
        (16, 10),
        (14, 10),
        (2, 10),
        (10, 10),
        (13, 10),
        (7, 10),
        (19, 10),
        (5, 8),
    ]
    .into_iter()
    .map(|(price, discount)| BiddedCollateralItem {
        collection: Addr::unchecked("collection"),
        token_id: "42".to_string(),
        owner: Addr::unchecked("bob"),
        liquidator: Addr::unchecked("alice"),
        bid_creation_date: 0,
        liquidation_price: Uint128::new(price),
        collateral_price: (u128_to_dec(price)
            / (Decimal::one() - u128_to_dec(discount as u128) / u128_to_dec(100u128)))
        .to_uint_ceil(),
        bid_discount: u128_to_dec(discount as u128),
        is_offer: false,
    })
    .collect();

    let borrow_apr = str_to_dec("0.25");
    let discount_max_rate = str_to_dec("0.2");
    let accumulated_loan = Uint128::zero();
    let loan = Uint128::new(70);
    let block_time = 0;
    let loan_creation_date = 0;
    let collateral = collateral_list
        .iter()
        .fold(Uint128::zero(), |acc, cur| acc + cur.collateral_price);

    let ltv = math::calc_conditional_ltv(
        borrow_apr,
        Uint128::zero(),
        accumulated_loan,
        loan,
        Uint128::zero(),
        collateral,
        block_time,
        loan_creation_date,
    );

    let ltv_max = str_to_dec("0.5");

    let max_liquidation_value =
        math::calc_liquidation_value(collateral, discount_max_rate, ltv, ltv_max);

    let liquidation_set = math::calc_liquidation_set(
        &collateral_list,
        max_liquidation_value,
        ltv_max,
        borrow_apr,
        accumulated_loan,
        loan,
        collateral,
        block_time,
        loan_creation_date,
    );

    assert_that(&liquidation_set).is_equal_to(vec![
        BiddedCollateralItem {
            collection: Addr::unchecked("collection"),
            token_id: "42".to_string(),
            owner: Addr::unchecked("bob"),
            liquidator: Addr::unchecked("alice"),
            bid_creation_date: 0,
            liquidation_price: Uint128::new(19),
            collateral_price: Uint128::new(22),
            bid_discount: u128_to_dec(10u128),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: Addr::unchecked("collection"),
            token_id: "42".to_string(),
            owner: Addr::unchecked("bob"),
            liquidator: Addr::unchecked("alice"),
            bid_creation_date: 0,
            liquidation_price: Uint128::new(16),
            collateral_price: Uint128::new(18),
            bid_discount: u128_to_dec(10u128),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: Addr::unchecked("collection"),
            token_id: "42".to_string(),
            owner: Addr::unchecked("bob"),
            liquidator: Addr::unchecked("alice"),
            bid_creation_date: 0,
            liquidation_price: Uint128::new(5),
            collateral_price: Uint128::new(6),
            bid_discount: u128_to_dec(8u128),
            is_offer: false,
        },
    ]);
}

#[test]
fn match_collaterals_and_bids_and_calc_liquidation_sets_default() -> StdResult<()> {
    let collection_and_collateral_list_by_borrower: Vec<(Addr, Collateral)> = vec![
        (
            ProjectNft::Gopniks.into(),
            Collateral {
                owner: ProjectAccount::Owner.into(),
                token_id_list: to_string_vec(vec!["1", "7", "4"]),
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            Collateral {
                owner: ProjectAccount::Owner.into(),
                token_id_list: to_string_vec(vec!["1", "8", "2", "5"]),
            },
        ),
    ];

    let price_list: Vec<PriceItem> = vec![
        PriceItem {
            address: ProjectNft::Gopniks.into(),
            price: Funds::new(
                Uint128::new(200),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
            price_update_date: 0,
        },
        PriceItem {
            address: ProjectNft::Pinjeons.into(),
            price: Funds::new(
                Uint128::new(100),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
            price_update_date: 1_000,
        },
    ];

    let collection_and_bid_list: Vec<(Addr, LiquidationBid)> = vec![
        // gopnics
        // too small
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 100,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["1"]),
                amount: Uint128::new(75),
                discount: str_to_dec("0"),
                is_offer: false,
            },
        ),
        // out of scope
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 100,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["42"]),
                amount: Uint128::new(750),
                discount: str_to_dec("0"),
                is_offer: false,
            },
        ),
        // 2 tokens bid, 1 token funds
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["1", "4"]),
                amount: Uint128::new(300),
                discount: str_to_dec("0.1"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["1", "4"]),
                amount: Uint128::new(300),
                discount: str_to_dec("0.05"),
                is_offer: false,
            },
        ),
        // offer
        (
            ProjectNft::Gopniks.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["7"]),
                amount: Uint128::new(900),
                discount: str_to_dec("0"),
                is_offer: true,
            },
        ),
        // pingeons
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["2"]),
                amount: Uint128::new(150),
                discount: str_to_dec("0.1"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["2"]),
                amount: Uint128::new(125),
                discount: str_to_dec("0.1"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["5"]),
                amount: Uint128::new(125),
                discount: str_to_dec("0"),
                is_offer: true,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Bob.into(),
                token_id_list: to_string_vec(vec!["1", "8", "42"]),
                amount: Uint128::new(125),
                discount: str_to_dec("0.05"),
                is_offer: false,
            },
        ),
        (
            ProjectNft::Pinjeons.into(),
            LiquidationBid {
                creation_date: 500,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(vec!["8", "42"]),
                amount: Uint128::new(225),
                discount: str_to_dec("0.2"),
                is_offer: false,
            },
        ),
    ];

    let collaterals_and_bids = math::match_collaterals_and_bids(
        &collection_and_collateral_list_by_borrower,
        &collection_and_bid_list,
        &price_list,
    )?;

    let borrow_apr = str_to_dec("0.25");
    let discount_max_rate = str_to_dec("0.2");
    let accumulated_loan = Uint128::zero();
    let loan = Uint128::new(600);
    let block_time = 0;
    let loan_creation_date = 0;
    let collateral = Uint128::new(700);

    let ltv = math::calc_conditional_ltv(
        borrow_apr,
        Uint128::zero(),
        accumulated_loan,
        loan,
        Uint128::zero(),
        collateral,
        block_time,
        loan_creation_date,
    );

    let ltv_max = str_to_dec("0.5");

    let max_liquidation_value =
        math::calc_liquidation_value(collateral, discount_max_rate, ltv, ltv_max);

    let liquidation_set = math::calc_liquidation_set(
        &collaterals_and_bids,
        max_liquidation_value,
        ltv_max,
        borrow_apr,
        accumulated_loan,
        loan,
        collateral,
        block_time,
        loan_creation_date,
    );

    assert_that(&liquidation_set).is_equal_to(vec![
        BiddedCollateralItem {
            collection: ProjectNft::Gopniks.into(),
            token_id: "1".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Bob.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(190),
            collateral_price: Uint128::new(200),
            bid_discount: str_to_dec("0.05"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Gopniks.into(),
            token_id: "4".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(180),
            collateral_price: Uint128::new(200),
            bid_discount: str_to_dec("0.1"),
            is_offer: false,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Pinjeons.into(),
            token_id: "5".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(100),
            collateral_price: Uint128::new(100),
            bid_discount: str_to_dec("0"),
            is_offer: true,
        },
        BiddedCollateralItem {
            collection: ProjectNft::Pinjeons.into(),
            token_id: "8".to_string(),
            owner: ProjectAccount::Owner.into(),
            liquidator: ProjectAccount::Alice.into(),
            bid_creation_date: 500,
            liquidation_price: Uint128::new(80),
            collateral_price: Uint128::new(100),
            bid_discount: str_to_dec("0.2"),
            is_offer: false,
        },
    ]);

    Ok(())
}
