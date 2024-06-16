use cw_multi_test::Executor;
use speculoos::assert_that;

use cosmwasm_std::{StdResult, Uint128};

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    error::ContractError,
    oracle::{
        msg::MigrateMsg,
        types::{PriceItem, RawPriceItem},
    },
};

use crate::helpers::{
    oracle::OracleExtension,
    suite::{
        core::{assert_error, Project},
        types::{GetDecimals, ProjectAccount, ProjectCoin, ProjectNft},
    },
};

#[test]
fn default() -> StdResult<()> {
    let mut project = Project::new();

    let block_time = project.get_block_time();

    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection_address: ProjectNft::Pinjeons.to_string(),
                price: Funds::new(
                    Uint128::new(300),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                ),
                usd_price: Funds::new(
                    Uint128::new(300),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                        ProjectCoin::Usdc.get_decimals(),
                    ),
                ),
            },
            RawPriceItem {
                collection_address: ProjectNft::Gopniks.to_string(),
                price: Funds::new(
                    Uint128::new(800),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                ),
                usd_price: Funds::new(
                    Uint128::new(800),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                        ProjectCoin::Usdc.get_decimals(),
                    ),
                ),
            },
        ],
    )?;

    let res = project.oracle_query_prices(&None, &None, &None)?;

    project.wait(3);

    assert_that(&res).is_equal_to(vec![
        PriceItem {
            address: ProjectNft::Gopniks.into(),
            price: Funds::new(
                Uint128::new(800),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            usd_price: Funds::new(
                Uint128::new(800),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Usdc.to_string()),
                    ProjectCoin::Usdc.get_decimals(),
                ),
            ),
            price_update_date: block_time,
        },
        PriceItem {
            address: ProjectNft::Pinjeons.into(),
            price: Funds::new(
                Uint128::new(300),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            usd_price: Funds::new(
                Uint128::new(300),
                &Currency::new(
                    &Token::new_native(&ProjectCoin::Usdc.to_string()),
                    ProjectCoin::Usdc.get_decimals(),
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
            &[
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    price: Funds::new(
                        Uint128::new(300),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                            ProjectCoin::Stars.get_decimals(),
                        ),
                    ),
                    usd_price: Funds::new(
                        Uint128::new(300),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                            ProjectCoin::Usdc.get_decimals(),
                        ),
                    ),
                },
                RawPriceItem {
                    collection_address: ProjectNft::Gopniks.to_string(),
                    price: Funds::new(
                        Uint128::new(800),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                            ProjectCoin::Stars.get_decimals(),
                        ),
                    ),
                    usd_price: Funds::new(
                        Uint128::new(800),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                            ProjectCoin::Usdc.get_decimals(),
                        ),
                    ),
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
            ProjectAccount::Scheduler,
            &[
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    price: Funds::new(
                        Uint128::new(300),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                            ProjectCoin::Stars.get_decimals(),
                        ),
                    ),
                    usd_price: Funds::new(
                        Uint128::new(300),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                            ProjectCoin::Usdc.get_decimals(),
                        ),
                    ),
                },
                RawPriceItem {
                    collection_address: ProjectNft::Pinjeons.to_string(),
                    price: Funds::new(
                        Uint128::new(300),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                            ProjectCoin::Stars.get_decimals(),
                        ),
                    ),
                    usd_price: Funds::new(
                        Uint128::new(300),
                        &Currency::new(
                            &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                            ProjectCoin::Usdc.get_decimals(),
                        ),
                    ),
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
        .oracle_try_update_prices(ProjectAccount::Scheduler, &[])
        .unwrap_err();

    assert_error(&res, ContractError::EmptyRawPriceItemVector);

    Ok(())
}

#[test]
fn zeros_in_prices() -> StdResult<()> {
    let mut project = Project::new();

    let res = project
        .oracle_try_update_prices(
            ProjectAccount::Scheduler,
            &[RawPriceItem {
                collection_address: ProjectNft::Pinjeons.to_string(),
                price: Funds::new(
                    Uint128::new(0),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                ),
                usd_price: Funds::new(
                    Uint128::new(0),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                        ProjectCoin::Usdc.get_decimals(),
                    ),
                ),
            }],
        )
        .unwrap_err();

    assert_error(&res, ContractError::ZerosInPrices);

    Ok(())
}

#[test]
fn migrate_default() {
    let mut project = Project::new();

    project
        .app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            project.get_oracle_address(),
            &MigrateMsg {
                version: "1.0.0".to_string(),
            },
            project.get_oracle_code_id(),
        )
        .unwrap();
}
