use goplend_base::{
    assets::{Currency, Funds, TokenUnverified},
    converters::str_to_dec,
    error::ContractError,
    lending_platform::{
        msg::{QueryCollateralsResponseItem, QueryLiquidationBidsResponseItem},
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

use cosmwasm_std::{Decimal, StdResult, Uint128};

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    oracle::OracleExtension,
    suite::{
        core::{assert_error, Project, INITIAL_LIQUIDITY},
        types::{GetDecimals, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
    },
};

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
    let config = project.lending_platform_query_config()?;
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

    assert_that(&balances.total_deposited.u128()).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&balances.total_main_currency_pool.u128())
        .is_equal_to(INITIAL_LIQUIDITY + DEPOSIT_AMOUNT);
    assert_that(&balances.total_bgl_currency_supply.u128())
        .is_equal_to(INITIAL_LIQUIDITY + DEPOSIT_AMOUNT);

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
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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
    project.lending_platform_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(1_200_000_000),
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
    let config = project.lending_platform_query_config()?;
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

    assert_that(&balances.total_deposited.u128()).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&balances.total_unbonded.u128()).is_equal_to(0);
    assert_that(&balances.total_main_currency_pool.u128()).is_equal_to(INITIAL_LIQUIDITY);
    assert_that(&balances.total_bgl_currency_supply.u128()).is_equal_to(INITIAL_LIQUIDITY);

    Ok(())
}

#[test]
fn min_available_liquidity_limitation_without_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 100_000_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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
    // 100_000_000_000 - 0 - 0 = 100_000_000_000
    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(100_000_000_000);

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT - 1)?;

    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn min_available_liquidity_limitation_with_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 1_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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
    project.lending_platform_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(1),
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

    // min_available_liquidity = 0.15 * (total_main_currency_pool + accumulated_expected_rewards - reserve_main_currency_pool)
    // total_main_currency_pool - total_borrowed_amount - min_available_liquidity
    // min_available_liquidity = 0.15 * 500_000
    // 100_000_000_000 + 500_000 - 0 - 0.15 * 500_000 = 100_000_425_000
    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(100_000_425_000);

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, 100_000_450_000)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, 100_000_425_000 - 1)?;

    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn clear_unbonders_storage() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_config()?;
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

    let unbonders = project.lending_platform_query_unbonders(&None)?;
    assert_that(&unbonders.len()).is_equal_to(1);

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    let unbonders = project.lending_platform_query_unbonders(&None)?;
    assert_that(&unbonders.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_collaterals_storage() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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

    let collaterals = project.lending_platform_query_collaterals(&None)?;
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

    let collaterals = project.lending_platform_query_collaterals(&None)?;
    assert_that(&collaterals.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_borrowers_storage() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 1_700;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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

    let borrowers = project.lending_platform_query_borrowers(&None)?;
    assert_that(&borrowers.len()).is_equal_to(1);

    project.lending_platform_try_repay(ProjectAccount::Bob, AMOUNT, ProjectCoin::Stars)?;

    let borrowers = project.lending_platform_query_borrowers(&None)?;
    assert_that(&borrowers.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_liquidators_and_bids_storage() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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

    let liquidators = project.lending_platform_query_liquidators(&None)?;
    let bids = project.lending_platform_query_liquidation_bids_by_liquidator_addresses(&None)?;

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

    let liquidators = project.lending_platform_query_liquidators(&None)?;
    let bids = project.lending_platform_query_liquidation_bids_by_liquidator_addresses(&None)?;

    assert_that(&liquidators.is_empty()).is_equal_to(true);
    assert_that(&bids.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn process_default() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 700;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_500;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
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
    project.lending_platform_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(1),
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

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    let deposit_apr = project.lending_platform_query_deposit_apr()?;
    assert_that(&deposit_apr).is_equal_to(str_to_dec(MAX_DEPOSIT_APR));

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Stars,
    )?;

    let deposit_apr = project.lending_platform_query_deposit_apr()?;
    assert_that(&deposit_apr).is_equal_to(str_to_dec(BORROW_APR) - str_to_dec(BORROW_FEE_RATE));

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: vec!["4", "5"].into_iter().map(|x| x.to_string()).collect(),
        }],
        "0",
        LIQUIDATION_BID,
        ProjectCoin::Stars,
    )?;

    project.wait(100);

    let collaterals_before = project.lending_platform_query_collaterals(&None)?;
    let liquidation_bids_before =
        project.lending_platform_query_liquidation_bids_by_collection_addresses(&None)?;

    // process without liquidations
    project.lending_platform_try_process()?;

    let collaterals_after = project.lending_platform_query_collaterals(&None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_addresses(&None)?;

    assert_that(&collaterals_before).is_equal_to(collaterals_after);
    assert_that(&liquidation_bids_before).is_equal_to(liquidation_bids_after);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_AFTER),
                &Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
        }],
    )?;

    // process with liquidations
    project.lending_platform_try_process()?;

    let ltv_after = project.lending_platform_query_ltv(ProjectAccount::Bob)?;
    let collaterals_after = project.lending_platform_query_collaterals(&None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_addresses(&None)?;

    let ltv_max = calc_ltv_max(
        str_to_dec(BID_MIN_RATE),
        str_to_dec(DISCOUNT_MAX_RATE),
        str_to_dec(DISCOUNT_MIN_RATE),
    );

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_id_list: vec!["5", "6"].into_iter().map(|x| x.to_string()).collect(),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_id_list: vec!["4"].into_iter().map(|x| x.to_string()).collect(),
            },
        ],
    }];

    let expected_liquidation_bids_after = vec![QueryLiquidationBidsResponseItem {
        address: ProjectNft::Gopniks.into(),
        liquidation_bids: vec![LiquidationBid {
            creation_date: 1571797419,
            liquidator: ProjectAccount::Alice.into(),
            token_id_list: vec!["5"].into_iter().map(|x| x.to_string()).collect(),
            amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
            discount: Decimal::zero(),
        }],
    }];

    assert_that(&(ltv_after <= ltv_max)).is_equal_to(true);
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}
