use goplend_base::{
    assets::{Currency, Token, TokenUnverified},
    converters::str_to_dec,
    error::ContractError,
};
use speculoos::assert_that;

use cosmwasm_std::StdResult;

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    suite::{
        core::{assert_error, Project, INITIAL_LIQUIDITY},
        types::{GetDecimals, ProjectAccount, ProjectCoin, ProjectToken},
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
fn scheduler_for_lending_platform() -> StdResult<()> {
    let mut project = Project::new();
    project.reset_time();

    project.lending_platform_try_process()?;
    let res = project.lending_platform_query_block_counter()?;
    assert_that(&res).is_equal_to(200);

    project.wait(10);

    project.lending_platform_try_process()?;
    let res = project.lending_platform_query_block_counter()?;
    assert_that(&res).is_equal_to(202);

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
