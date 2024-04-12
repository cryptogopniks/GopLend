use goplend_base::{
    adapters::scheduler::kujira::types::ClockSource,
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::{str_to_dec, u128_to_dec},
    error::ContractError,
    lending_platform::{
        msg::{
            QueryCollateralsResponseItem, QueryLiquidationBidsByCollectionAddressListResponseItem,
        },
        state::{
            BID_MIN_RATE, BORROW_APR, BORROW_FEE_RATE, DISCOUNT_MAX_RATE, DISCOUNT_MIN_RATE,
            MAX_DEPOSIT_APR,
        },
        types::{Collateral, Collection, CollectionInfo, LiquidationBid, Proposal, ProposalType},
    },
    oracle::types::RawPriceItem,
};
use lending_platform::math::calc_ltv_max;
use speculoos::assert_that;

use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};

use crate::helpers::{
    adapters::scheduler::kujira::AdapterSchedulerKujiraExtension,
    lending_platform::LendingPlatformExtension,
    oracle::OracleExtension,
    suite::{
        core::{assert_error, Project, INITIAL_LIQUIDITY},
        types::{GetDecimals, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
    },
};

#[test]
fn deposit_apr_with_depositors_and_borrowers() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const COLLECTION_PRICE_AFTER: u128 = 600;
    const AMOUNT: u128 = 1_000_000;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &None,
        &None,
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Stars)?;

    // check initial deposit apr and bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    // check deposit apr and bgl price after 1/2 year
    project.wait(DELAY / 2);

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    // borrow and check deposit apr and bgl price
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.2"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    // check deposit apr and bgl price after 1/2 year
    project.wait(DELAY / 2);

    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.2"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1.1"));

    Ok(())
}

#[test]
fn deposit_apr_without_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const COLLECTION_PRICE_AFTER: u128 = 600;
    const AMOUNT: u128 = 1_000_000;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &None,
        &None,
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    // check initial deposit apr and bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec(MAX_DEPOSIT_APR));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    Ok(())
}

#[test]
fn borrow_and_repay_all_after_1_year() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const COLLECTION_PRICE_AFTER: u128 = 600;
    const AMOUNT: u128 = 1_000_000;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &None,
        &None,
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Stars)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    // check accumulated loan
    project.wait(DELAY);

    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    assert_that(&borrower.accumulated_loan.u128()).is_equal_to(250_000);

    // repay all
    project.lending_platform_try_repay(ProjectAccount::Bob, 1_250_000, ProjectCoin::Stars)?;

    // check loans
    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    assert_that(&borrower.loan.amount.u128()).is_equal_to(0);
    assert_that(&borrower.accumulated_loan.u128()).is_equal_to(0);

    // check apr, bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1.2"));

    // check platform revenue
    let revenue = project.lending_platform_query_platform_revenue()?;

    assert_that(
        &revenue
            .iter()
            .find(|x| x.currency.token == ProjectCoin::Stars.into())
            .unwrap()
            .amount
            .u128(),
    )
    .is_equal_to(50_000);

    Ok(())
}

#[test]
fn multiple_borrow_and_repay() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const COLLECTION_PRICE_AFTER: u128 = 600;
    const AMOUNT: u128 = 1_000_000;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection_address: ProjectNft::Gopniks.to_string(),
                price: Funds::new(
                    Uint128::new(COLLECTION_PRICE_BEFORE),
                    &Currency::new(
                        &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                        ProjectToken::Atom.get_decimals(),
                    ),
                ),
            },
            RawPriceItem {
                collection_address: ProjectNft::Pinjeons.to_string(),
                price: Funds::new(
                    Uint128::new(COLLECTION_PRICE_BEFORE),
                    &Currency::new(
                        &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                        ProjectToken::Atom.get_decimals(),
                    ),
                ),
            },
        ],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &Some(DELAY),
        &None,
    )?;

    // list collection
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral
    for user in vec![ProjectAccount::Bob, ProjectAccount::John] {
        project.increase_allowances_nft(
            user,
            project.get_lending_platform_address(),
            ProjectNft::Gopniks,
        );
        project.increase_allowances_nft(
            user,
            project.get_lending_platform_address(),
            ProjectNft::Pinjeons,
        );
    }

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: vec!["4", "5", "6"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: vec!["4", "5", "6"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
        ],
    )?;
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::John,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: vec!["7", "8", "9"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: vec!["7", "8", "9"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
        ],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Stars)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;
    project.lending_platform_try_borrow(ProjectAccount::John, AMOUNT / 2)?;

    // wait 1/4 year and borrow again
    project.wait(DELAY / 4);

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT / 2)?;
    project.lending_platform_try_borrow(ProjectAccount::John, AMOUNT)?;

    // wait 1/4 year and check loans
    project.wait(DELAY / 4);

    let borrower_bob = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let borrower_john = project.lending_platform_query_borrower(ProjectAccount::John)?;

    //  0.25 * (1_000_000 / 2 + 500_000 / 4) = 156_250
    assert_that(&borrower_bob.accumulated_loan.u128()).is_equal_to(156_250);
    assert_that(&borrower_bob.loan.amount.u128()).is_equal_to(1_500_000);
    //  0.25 * (500_000 / 2 + 1_000_000 / 4) = 125_000
    assert_that(&borrower_john.accumulated_loan.u128()).is_equal_to(125_000);
    assert_that(&borrower_john.loan.amount.u128()).is_equal_to(1_500_000);

    // repay half
    project.lending_platform_try_repay(
        ProjectAccount::Bob,
        (borrower_bob.accumulated_loan + borrower_bob.loan.amount).u128() / 2,
        ProjectCoin::Stars,
    )?;
    project.lending_platform_try_repay(
        ProjectAccount::John,
        (borrower_john.accumulated_loan + borrower_john.loan.amount).u128() / 2,
        ProjectCoin::Stars,
    )?;

    // check loans
    let borrower_bob = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let borrower_john = project.lending_platform_query_borrower(ProjectAccount::John)?;

    //  1_500_000 - (828_125 - 156_250) = 828_125
    assert_that(&borrower_bob.accumulated_loan.u128()).is_equal_to(0);
    assert_that(&borrower_bob.loan.amount.u128()).is_equal_to(828_125);
    //  1_500_000 - (812_500 - 125_000) = 812_500
    assert_that(&borrower_john.accumulated_loan.u128()).is_equal_to(0);
    assert_that(&borrower_john.loan.amount.u128()).is_equal_to(812_500);

    // check apr, bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    // 0.2 * (828_125 + 812_500) / 1_000_000 = 0.328125
    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.328125"));
    // (1_000_000 + 0.2 * (1_500_000 / 2 + 1_500_000 / 4)) / 1_000_000 = 1.225
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1.225"));

    // wait 1/4 year and check loans
    project.wait(DELAY / 4);

    let borrower_bob = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let borrower_john = project.lending_platform_query_borrower(ProjectAccount::John)?;

    assert_that(&borrower_bob.accumulated_loan.u128()).is_equal_to(51_758);
    assert_that(&borrower_bob.loan.amount.u128()).is_equal_to(828_125);
    assert_that(&borrower_john.accumulated_loan.u128()).is_equal_to(50_781);
    assert_that(&borrower_john.loan.amount.u128()).is_equal_to(812_500);

    // check platform revenue
    let revenue = project.lending_platform_query_platform_revenue()?;

    assert_that(
        &revenue
            .iter()
            .find(|x| x.currency.token == ProjectCoin::Stars.into())
            .unwrap()
            .amount
            .u128(),
    )
    .is_equal_to(56_250);

    Ok(())
}

// #[test]
// fn price_ratio_with_different_decimals() -> StdResult<()> {
//     let mut project = Project::new();

//     const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000;

//     project
//         .lending_platform_try_update_config(
//             ProjectAccount::Admin,
//             &None,
//             &None,
//             &Some(Currency::new(
//                 &TokenUnverified::new_cw20(ProjectToken::Inj.into()),
//                 ProjectToken::Inj.get_decimals(),
//             )),
//             &None,
//             &None,
//             &None,
//             &None,
//             &None,
//             &None,
//             &None,
//             &None,
//             &None,
//         )
//         .unwrap();

//     let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
//     assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

//     project.lending_platform_try_deposit(
//         ProjectAccount::Alice,
//         DEPOSIT_AMOUNT,
//         ProjectToken::Inj,
//     )?;

//     let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
//     assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

//     let balances = project.lending_platform_query_balances()?;
//     assert_that(&(balances.total_main_currency_pool / balances.total_bgl_currency_supply).u128())
//         .is_equal_to(1_000_000_000_000);

//     Ok(())
// }

#[test]
fn deposit_improper_main_currency() -> StdResult<()> {
    let mut project = Project::new();

    const DEPOSIT_AMOUNT: u128 = 1_000;

    // deposit improper token
    let res = project
        .lending_platform_try_deposit(ProjectAccount::Alice, DEPOSIT_AMOUNT, ProjectCoin::Noria)
        .unwrap_err();

    assert_error(&res, ContractError::WrongAssetType);

    Ok(())
}

#[test]
fn deposit_zero() -> StdResult<()> {
    let mut project = Project::new();

    const DEPOSIT_AMOUNT: u128 = 0;

    let res = project
        .lending_platform_try_deposit(ProjectAccount::Alice, DEPOSIT_AMOUNT, ProjectCoin::Stars)
        .unwrap_err();

    assert_error(&res, "Cannot transfer empty coins amount");

    Ok(())
}

#[test]
fn deposit_default() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Stars)?;
    let bgl_stars_before = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Stars,
    )?;

    // check user balances
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Stars)?;
    let bgl_stars_after = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    let stars_difference = stars_before - stars_after;
    let bgl_stars_difference = bgl_stars_after - bgl_stars_before;

    assert_that(&stars_difference).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&bgl_stars_difference).is_equal_to(DEPOSIT_AMOUNT);

    // check contract balances
    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.deposited.u128()).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&balances.regular.bgl_supply.u128()).is_equal_to(DEPOSIT_AMOUNT);

    Ok(())
}

#[test]
fn deposit_does_not_change_bgl_price() -> StdResult<()> {
    let mut project = Project::new();

    const DEPOSIT_AMOUNT: u128 = 1_000;

    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Stars,
    )?;

    // deposit doesn't change bgl token price
    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

    project.lending_platform_try_deposit(
        ProjectAccount::Bob,
        DEPOSIT_AMOUNT,
        ProjectCoin::Stars,
    )?;

    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

    Ok(())
}

#[test]
fn collection_address_duplication() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(800_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[
                CollectionInfo {
                    collection_address: ProjectNft::Gopniks.to_string(),
                    token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
                },
                CollectionInfo {
                    collection_address: ProjectNft::Gopniks.to_string(),
                    token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
                },
            ],
        )
        .unwrap_err();

    assert_error(&res, ContractError::CollectionDuplication);

    Ok(())
}

#[test]
fn collateral_value_limitations() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(800_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1_200_000_000),
        &None,
        &None,
        &None,
    )?;

    // collateral value = 800_000_000
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
            }],
        )
        .unwrap_err();

    assert_error(&res, "min collateral");

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
        }],
    )?;

    let res = project
        .lending_platform_try_withdraw_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
            }],
        )
        .unwrap_err();

    assert_error(&res, "min collateral");

    project.lending_platform_try_withdraw_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
        }],
    )?;

    Ok(())
}

#[test]
fn deposit_unbond_withdraw_default() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Stars)?;
    let bgl_stars_before = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Stars,
    )?;

    // check contract balances
    let balances = project.lending_platform_query_balances()?;
    assert_that(&balances.regular.deposited.u128()).is_equal_to(DEPOSIT_AMOUNT);

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // check user balances
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Stars)?;
    let bgl_stars_after = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    let stars_difference = stars_before - stars_after;
    let bgl_stars_difference = bgl_stars_after - bgl_stars_before;

    assert_that(&stars_difference).is_equal_to(0);
    assert_that(&bgl_stars_difference).is_equal_to(0);

    // check contract balances
    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.deposited.u128()).is_equal_to(0);
    assert_that(&balances.regular.unbonded.u128()).is_equal_to(0);
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(0);
    assert_that(&balances.regular.bgl_supply.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn min_available_liquidity_limitation_without_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 85_000_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(80_000_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    // total_main_currency_pool - total_borrowed_amount - min_available_liquidity
    // 100_000_000_000 - 0 - 0.15 * 100_000_000_000 = 85_000_000_000
    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(AMOUNT);

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT + 1)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn min_available_liquidity_limitation_with_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 1_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(80_000_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &None,
        &None,
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT / 2, ProjectCoin::Stars)?;

    // min_available_liquidity = 0.15 * reserve_deposited
    // 0.15 * 100_000_000_000 = 15_000_000_000
    // regular_available = regular_deposited + acc_rewards - regular_unbonded - regular_borrowed
    // 500_000 + 0 - 0 - 0 = 500_000
    // reserve_available = reserve_deposited - min_available_liquidity - reserve_borrowed
    // 100_000_000_000 - 15_000_000_000 - 0 = 85_000_000_000
    // total_available_liquidity = regular_available + reserve_available
    // 500_000 + 85_000_000_000 = 85_000_500_000
    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(85_000_500_000);

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, 85_000_500_001)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, 85_000_500_000)?;

    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn clear_unbonders_storage() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Stars,
    )?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT, &bgl_token)?;

    let unbonders = project.lending_platform_query_unbonder_list(&None, &None)?;
    assert_that(&unbonders.len()).is_equal_to(1);

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    let unbonders = project.lending_platform_query_unbonder_list(&None, &None)?;
    assert_that(&unbonders.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_collaterals_storage() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(80_000_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    let collaterals = project.lending_platform_query_collateral_list(&None, &None)?;
    assert_that(&collaterals.len()).is_equal_to(1);

    project.lending_platform_try_withdraw_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    let collaterals = project.lending_platform_query_collateral_list(&None, &None)?;
    assert_that(&collaterals.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_borrowers_storage() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 1_700;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(80_000_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    // TODO
    // let borrowers = project.lending_platform_query_borrower_list(&None, &None)?;
    // assert_that(&borrowers.len()).is_equal_to(1);

    // project.lending_platform_try_repay(ProjectAccount::Bob, AMOUNT, ProjectCoin::Stars)?;

    // let borrowers = project.lending_platform_query_borrower_list(&None, &None)?;
    // assert_that(&borrowers.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_liquidators_and_bids_storage() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(80_000_000_000),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
        }],
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5", "6"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        }],
    )?;

    let block_time = project.get_timestamp().seconds();
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
        }],
        "0",
        800,
        ProjectCoin::Stars,
    )?;

    let liquidators = project.lending_platform_query_liquidator_list(&None, &None)?;
    let bids =
        project.lending_platform_query_liquidation_bids_by_liquidator_address_list(&None, &None)?;

    assert_that(&liquidators.len()).is_equal_to(1);
    assert_that(&bids.len()).is_equal_to(1);

    project.lending_platform_try_remove_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
        }],
        block_time,
    )?;

    let liquidators = project.lending_platform_query_liquidator_list(&None, &None)?;
    let bids =
        project.lending_platform_query_liquidation_bids_by_liquidator_address_list(&None, &None)?;

    assert_that(&liquidators.is_empty()).is_equal_to(true);
    assert_that(&bids.is_empty()).is_equal_to(true);

    Ok(())
}

// #[test]
// fn partial_liquidation_single_liquidator_enough_bids_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 700;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_500;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
//     assert_that(&deposit_apr).is_equal_to(str_to_dec(MAX_DEPOSIT_APR));

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None)?;
//     assert_that(&deposit_apr).is_equal_to(str_to_dec(BORROW_APR) - str_to_dec(BORROW_FEE_RATE));

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     let collaterals_before = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_before =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     // process without liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     assert_that(&collaterals_before).is_equal_to(collaterals_after);
//     assert_that(&liquidation_bids_before).is_equal_to(liquidation_bids_after);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["5"].into_iter().map(|x| x.to_string()).collect(),
//                 amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after <= ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn partial_liquidation_single_liquidator_enough_bids_not_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 650;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_200;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//                 amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after > ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn partial_liquidation_single_liquidator_not_enough_bids_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 650;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_400;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec![],
//                 amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after > ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn partial_liquidation_single_liquidator_not_enough_bids_not_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 650;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_200;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec![],
//                 amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after > ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn full_liquidation_single_liquidator_enough_bids_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 400;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_300;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after = project
//         .lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)
//         .unwrap_err();
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![Collateral {
//             owner: ProjectAccount::Alice.into(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec![],
//                 amount: Uint128::new(LIQUIDATION_BID - 3 * COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_error(&ltv_after, ContractError::CollateralIsNotFound);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn full_liquidation_single_liquidator_enough_bids_not_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 400;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_100;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//                 amount: Uint128::new(LIQUIDATION_BID - 2 * COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after > ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn full_liquidation_single_liquidator_not_enough_bids_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 400;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_300;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec![],
//                 amount: Uint128::new(LIQUIDATION_BID - 2 * COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after > ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn full_liquidation_single_liquidator_not_enough_bids_not_enough_funds() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 400;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 1_100;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;

//     let ltv_max = calc_ltv_max(
//         str_to_dec(BID_MIN_RATE),
//         str_to_dec(DISCOUNT_MAX_RATE),
//         str_to_dec(DISCOUNT_MIN_RATE),
//     );

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec![],
//                 amount: Uint128::new(LIQUIDATION_BID - 2 * COLLECTION_PRICE_AFTER),
//                 discount: Decimal::zero(),
//                 is_offer: false,
//             }],
//         }];

//     assert_that(&(ltv_after > ltv_max)).is_equal_to(true);
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn single_discount_liquidation() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 650;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 2_000;
//     const DISCOUNT: &str = "0.05";

//     // 5 % discount
//     // need to liquidate 2 * 650 = 1_300
//     // liquidator will pay 2 * ceil(650 * 0.95) = 2 * 618 = 1_236
//     // ltv = (1_700 - 1_236) / (1_950 - 1_300) = 0.7138

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//         DISCOUNT,
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;

//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;
//     // let balances_after = project.lending_platform_query_balances()?;
//     // let platform_revenue = project.lending_platform_query_platform_revenue()?;

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//                 amount: Uint128::new(
//                     LIQUIDATION_BID
//                         - 2 * (u128_to_dec(COLLECTION_PRICE_AFTER)
//                             * (Decimal::one() - str_to_dec(DISCOUNT)))
//                         .to_uint_ceil()
//                         .u128(),
//                 ),
//                 discount: str_to_dec(DISCOUNT),
//                 is_offer: false,
//             }],
//         }];

//     // TODO
//     // println!("{:#?}", platform_revenue);
//     // println!("{:#?}", balances_after);

//     assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.713846153846153846");
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn multiple_discount_liquidation() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 680;
//     const AMOUNT_TO_BORROW: u128 = 1_700;
//     const LIQUIDATION_BID: u128 = 2_000;
//     const DISCOUNT_0: &str = "0";
//     const DISCOUNT_1: &str = "0.2";

//     // 20, 0 % discount
//     // liquidator will pay ceil(680 * 0.8) + ceil(680) = 1_224
//     // ltv = (1_700 - 1_224) / 680 = 0.7

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_BEFORE),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         DISCOUNT_1,
//         LIQUIDATION_BID / 2,
//         ProjectCoin::Stars,
//     )?;
//     project.wait(10);

//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["5"].into_iter().map(|x| x.to_string()).collect(),
//         }],
//         DISCOUNT_0,
//         LIQUIDATION_BID / 2,
//         ProjectCoin::Stars,
//     )?;
//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;
//     // let balances_after = project.lending_platform_query_balances()?;
//     // let platform_revenue = project.lending_platform_query_platform_revenue()?;

//     let expected_collateral = vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![
//             Collateral {
//                 owner: ProjectAccount::Bob.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//             },
//             Collateral {
//                 owner: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["5", "4"].into_iter().map(|x| x.to_string()).collect(),
//             },
//         ],
//     }];

//     let expected_liquidation_bids_after =
//         vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![
//                 LiquidationBid {
//                     creation_date: 1571797419,
//                     liquidator: ProjectAccount::Alice.into(),
//                     token_id_list: vec![],
//                     amount: Uint128::new(
//                         LIQUIDATION_BID / 2
//                             - (u128_to_dec(COLLECTION_PRICE_AFTER)
//                                 * (Decimal::one() - str_to_dec(DISCOUNT_1)))
//                             .to_uint_ceil()
//                             .u128(),
//                     ),
//                     discount: str_to_dec(DISCOUNT_1),
//                     is_offer: false,
//                 },
//                 LiquidationBid {
//                     creation_date: 1571797429,
//                     liquidator: ProjectAccount::Alice.into(),
//                     token_id_list: vec![],
//                     amount: Uint128::new(
//                         LIQUIDATION_BID / 2
//                             - (u128_to_dec(COLLECTION_PRICE_AFTER)
//                                 * (Decimal::one() - str_to_dec(DISCOUNT_0)))
//                             .to_uint_ceil()
//                             .u128(),
//                     ),
//                     discount: str_to_dec(DISCOUNT_0),
//                     is_offer: false,
//                 },
//             ],
//         }];

//     // TODO
//     // println!("{:#?}", platform_revenue);
//     // println!("{:#?}", balances_after);

//     assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.7");
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// #[test]
// fn multiple_collection_liquidation() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 800;
//     const COLLECTION_PRICE_AFTER: u128 = 600;
//     const AMOUNT_TO_BORROW: u128 = 3_400;
//     const LIQUIDATION_BID: u128 = 8_000;
//     const DISCOUNT: &str = "0.1";

//     // 10 % discount
//     // liquidator will pay ceil(800 * 0.9) = 720
//     // ltv = (3400 - 720) / (4500 - 800) = 0.724
//     // ltv = (3400 - 1440) / (4500 - 1600) = 0.675

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[
//             RawPriceItem {
//                 collection_address: ProjectNft::Gopniks.to_string(),
//                 price: Funds::new(
//                     Uint128::new(COLLECTION_PRICE_BEFORE),
//                     &Currency::new(
//                         &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                         ProjectToken::Atom.get_decimals(),
//                     ),
//                 ),
//             },
//             RawPriceItem {
//                 collection_address: ProjectNft::Pinjeons.to_string(),
//                 price: Funds::new(
//                     Uint128::new(COLLECTION_PRICE_BEFORE),
//                     &Currency::new(
//                         &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                         ProjectToken::Atom.get_decimals(),
//                     ),
//                 ),
//             },
//         ],
//     )?;

//     // set collateral min value
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection_address: ProjectNft::Pinjeons.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Pinjeons.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         2,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Pinjeons,
//     );

//     // collateral value = 3 * collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection_address: ProjectNft::Pinjeons.to_string(),
//             token_id_list: vec!["4", "5", "6"]
//                 .into_iter()
//                 .map(|x| x.to_string())
//                 .collect(),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Stars,
//     )?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[
//             CollectionInfo {
//                 collection_address: ProjectNft::Gopniks.to_string(),
//                 token_id_list: vec!["4", "5", "6"]
//                     .into_iter()
//                     .map(|x| x.to_string())
//                     .collect(),
//             },
//             CollectionInfo {
//                 collection_address: ProjectNft::Pinjeons.to_string(),
//                 token_id_list: vec!["4", "5", "6"]
//                     .into_iter()
//                     .map(|x| x.to_string())
//                     .collect(),
//             },
//         ],
//         DISCOUNT,
//         LIQUIDATION_BID,
//         ProjectCoin::Stars,
//     )?;
//     project.wait(100);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection_address: ProjectNft::Gopniks.to_string(),
//             price: Funds::new(
//                 Uint128::new(COLLECTION_PRICE_AFTER),
//                 &Currency::new(
//                     &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                     ProjectToken::Atom.get_decimals(),
//                 ),
//             ),
//         }],
//     )?;

//     // let ltv_before =
//     //     project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     // let collaterals_before = project.lending_platform_query_collateral_list(&None,&None)?;
//     // let liquidation_bids_before =
//     //     project.lending_platform_query_liquidation_bids_by_collection_address_list(&None,&None)?;

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(&None, &None)?
//         .into_iter()
//         .map(|x| x.address)
//         .collect::<Vec<Addr>>();
//     let targets = project.lending_platform_query_targets(&borrowers)?;

//     project.adapter_scheduler_kujira_try_push(
//         ProjectAccount::Scheduler,
//         ClockSource::Offchain,
//         &targets,
//     )?;

//     let ltv_after =
//         project.lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None)?;
//     let collaterals_after = project.lending_platform_query_collateral_list(&None, &None)?;
//     let liquidation_bids_after =
//         project.lending_platform_query_liquidation_bids_by_collection_address_list(&None, &None)?;
//     // let balances_after = project.lending_platform_query_balances()?;
//     // let platform_revenue = project.lending_platform_query_platform_revenue()?;

//     let expected_collateral = vec![
//         QueryCollateralsResponseItem {
//             address: ProjectNft::Gopniks.into(),
//             collateral: vec![
//                 Collateral {
//                     owner: ProjectAccount::Bob.into(),
//                     token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//                 },
//                 Collateral {
//                     owner: ProjectAccount::Alice.into(),
//                     token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
//                 },
//             ],
//         },
//         QueryCollateralsResponseItem {
//             address: ProjectNft::Pinjeons.into(),
//             collateral: vec![
//                 Collateral {
//                     owner: ProjectAccount::Bob.into(),
//                     token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//                 },
//                 Collateral {
//                     owner: ProjectAccount::Alice.into(),
//                     token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
//                 },
//             ],
//         },
//     ];

//     let expected_liquidation_bids_after = vec![
//         QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Gopniks.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
//                 amount: Uint128::new(
//                     LIQUIDATION_BID
//                         - (u128_to_dec(COLLECTION_PRICE_AFTER)
//                             * (Decimal::one() - str_to_dec(DISCOUNT)))
//                         .to_uint_ceil()
//                         .u128(),
//                 ),
//                 discount: str_to_dec(DISCOUNT),
//                 is_offer: false,
//             }],
//         },
//         QueryLiquidationBidsByCollectionAddressListResponseItem {
//             collection_address: ProjectNft::Pinjeons.into(),
//             liquidation_bids: vec![LiquidationBid {
//                 creation_date: 1571797419,
//                 liquidator: ProjectAccount::Alice.into(),
//                 token_id_list: vec!["6"].into_iter().map(|x| x.to_string()).collect(),
//                 amount: Uint128::new(
//                     LIQUIDATION_BID
//                         - 2 * (u128_to_dec(COLLECTION_PRICE_BEFORE)
//                             * (Decimal::one() - str_to_dec(DISCOUNT)))
//                         .to_uint_ceil()
//                         .u128(),
//                 ),
//                 discount: str_to_dec(DISCOUNT),
//                 is_offer: false,
//             }],
//         },
//     ];

//     assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.71");
//     assert_that(&collaterals_after).is_equal_to(expected_collateral);
//     assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

//     Ok(())
// }

// TODO: check priority
// TODO: check targeted liquidations
// TODO: check balances changes
// TODO: check marketplace interface
