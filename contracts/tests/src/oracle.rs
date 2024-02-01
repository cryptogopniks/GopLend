use speculoos::assert_that;

use cosmwasm_std::{StdResult, Uint128};

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    error::ContractError,
    oracle::{msg::QueryPricesResponseItem, types::RawPriceItem},
};

use crate::helpers::{
    oracle::OracleExtension,
    suite::{
        core::{assert_error, Project},
        types::{GetDecimals, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
    },
};

#[test]
fn default() -> StdResult<()> {
    let mut project = Project::new();

    let block_time = project.get_timestamp().seconds();

    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![
            RawPriceItem {
                collection_address: ProjectNft::Pinjeons.to_string(),
                currency: Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
                raw_prices: vec![100, 200, 300, 400, 500]
                    .into_iter()
                    .map(Uint128::new)
                    .collect(),
                trading_volume_per_day: Uint128::new(10_000),
            },
            RawPriceItem {
                collection_address: ProjectNft::Gopniks.to_string(),
                currency: Currency::new(
                    &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                    ProjectToken::Atom.get_decimals(),
                ),
                raw_prices: vec![600, 700, 800, 900, 1000]
                    .into_iter()
                    .map(Uint128::new)
                    .collect(),
                trading_volume_per_day: Uint128::new(10_000),
            },
        ],
    )?;

    let res = project.oracle_query_prices(&None)?;

    project.wait(3);

    assert_that(&res).is_equal_to(vec![
        QueryPricesResponseItem {
            address: ProjectNft::Gopniks.into(),
            price: Funds::new(
                Uint128::new(800),
                &Currency::new(
                    &Token::new_cw20(&ProjectToken::Atom.into()),
                    ProjectToken::Atom.get_decimals(),
                ),
            ),
            price_update_date: block_time,
        },
        QueryPricesResponseItem {
            address: ProjectNft::Pinjeons.into(),
            price: Funds::new(
                Uint128::new(300),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            price_update_date: block_time,
        },
    ]);

    Ok(())
}

#[test]
fn unauthorized() -> StdResult<()> {
    let mut project = Project::new();

    let res = project
        .oracle_try_update_prices(
            ProjectAccount::Admin,
            &vec![
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    currency: Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                    raw_prices: vec![100, 200, 300, 400, 500]
                        .into_iter()
                        .map(Uint128::new)
                        .collect(),
                    trading_volume_per_day: Uint128::new(10_000),
                },
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    currency: Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                    raw_prices: vec![100, 200, 300, 400, 500]
                        .into_iter()
                        .map(Uint128::new)
                        .collect(),
                    trading_volume_per_day: Uint128::new(10_000),
                },
            ],
        )
        .unwrap_err();

    assert_error(&res, ContractError::Unauthorized);

    Ok(())
}

#[test]
fn duplicated_collections() -> StdResult<()> {
    let mut project = Project::new();

    let res = project
        .oracle_try_update_prices(
            ProjectAccount::SchedulerForOracle,
            &vec![
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    currency: Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                    raw_prices: vec![100, 200, 300, 400, 500]
                        .into_iter()
                        .map(Uint128::new)
                        .collect(),
                    trading_volume_per_day: Uint128::new(10_000),
                },
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    currency: Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                    raw_prices: vec![100, 200, 300, 400, 500]
                        .into_iter()
                        .map(Uint128::new)
                        .collect(),
                    trading_volume_per_day: Uint128::new(10_000),
                },
            ],
        )
        .unwrap_err();

    assert_error(&res, ContractError::CollectionDuplication);

    Ok(())
}

#[test]
fn empty_raw_price_item_vec() -> StdResult<()> {
    let mut project = Project::new();

    let res = project
        .oracle_try_update_prices(ProjectAccount::SchedulerForOracle, &vec![])
        .unwrap_err();

    assert_error(&res, ContractError::EmptyRawPriceItemVector);

    Ok(())
}

#[test]
fn empty_price_vec() -> StdResult<()> {
    let mut project = Project::new();

    let res = project
        .oracle_try_update_prices(
            ProjectAccount::SchedulerForOracle,
            &vec![
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    currency: Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                    raw_prices: vec![].into_iter().map(Uint128::new).collect(),
                    trading_volume_per_day: Uint128::new(10_000),
                },
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    currency: Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                    raw_prices: vec![100, 200, 300, 400, 500]
                        .into_iter()
                        .map(Uint128::new)
                        .collect(),
                    trading_volume_per_day: Uint128::new(10_000),
                },
            ],
        )
        .unwrap_err();

    assert_error(&res, ContractError::EmptyPriceVector);

    Ok(())
}

#[test]
fn not_enough_prices() -> StdResult<()> {
    let mut project = Project::new();

    project.oracle_try_update_prices(
        ProjectAccount::SchedulerForOracle,
        &vec![RawPriceItem {
            collection_address: ProjectNft::Pinjeons.to_string(),
            currency: Currency::new(
                &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                ProjectCoin::Stars.get_decimals(),
            ),
            raw_prices: vec![100, 200, 300].into_iter().map(Uint128::new).collect(),
            trading_volume_per_day: Uint128::new(10_000),
        }],
    )?;

    let res = project.oracle_query_prices(&None)?;

    assert_that(&res[0].price.amount.u128()).is_equal_to(120);

    Ok(())
}

#[test]
fn too_much_prices() -> StdResult<()> {
    let mut project = Project::new();

    let res = project
        .oracle_try_update_prices(
            ProjectAccount::SchedulerForOracle,
            &vec![RawPriceItem {
                collection_address: ProjectNft::Pinjeons.to_string(),
                currency: Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
                raw_prices: vec![100, 200, 300, 400, 500, 600, 700]
                    .into_iter()
                    .map(Uint128::new)
                    .collect(),
                trading_volume_per_day: Uint128::new(10_000),
            }],
        )
        .unwrap_err();

    assert_error(&res, ContractError::ExceededPricesMaxAmount);

    Ok(())
}
