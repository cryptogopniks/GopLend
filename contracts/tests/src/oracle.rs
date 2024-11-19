use cw_multi_test::Executor;
use speculoos::assert_that;

use cosmwasm_std::StdResult;

use goplend_base::{
    error::ContractError,
    oracle::{
        msg::MigrateMsg,
        state::DECIMALS_MULTIPLIER,
        types::{split_address, PriceItem},
    },
};

use crate::helpers::{
    oracle::OracleExtension,
    suite::{
        core::{assert_error, Project},
        types::{ProjectAccount, ProjectNft},
    },
};

pub fn to_prices_in(list: &[(ProjectNft, u32)]) -> Vec<(String, u32)> {
    list.iter()
        .map(|(collection, price)| (split_address(collection).1, *price))
        .collect()
}

pub fn to_prices_out(list: &[(ProjectNft, u128)]) -> Vec<PriceItem> {
    let mut prices: Vec<PriceItem> = list
        .iter()
        .map(|(collection, price)| PriceItem::new(collection, *price * DECIMALS_MULTIPLIER))
        .collect();
    prices.sort_unstable_by_key(|x| x.collection.clone());

    prices
}

#[test]
fn default() -> StdResult<()> {
    let mut p = Project::new();

    p.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Gopniks, 800), (ProjectNft::Pinjeons, 300)]),
    )?;

    let res = p.oracle_query_prices(9, &None, &None)?;

    p.wait(3);

    assert_that(&res.data).is_equal_to(to_prices_out(&[
        (ProjectNft::Gopniks, 800),
        (ProjectNft::Pinjeons, 300),
    ]));

    Ok(())
}

#[test]
fn unauthorized() -> StdResult<()> {
    let mut p = Project::new();

    let res = p
        .oracle_try_update_prices(
            ProjectAccount::Admin,
            &to_prices_in(&[(ProjectNft::Gopniks, 800), (ProjectNft::Pinjeons, 300)]),
        )
        .unwrap_err();

    assert_error(&res, ContractError::Unauthorized);

    Ok(())
}

// the guard was removed
#[test]
fn duplicated_collections() -> StdResult<()> {
    let mut p = Project::new();

    let _res = p.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Pinjeons, 300), (ProjectNft::Pinjeons, 300)]),
    )?;

    // assert_error(&res, ContractError::CollectionDuplication);

    Ok(())
}

// the guard was removed
#[test]
fn empty_raw_price_item_vec() -> StdResult<()> {
    let mut p = Project::new();

    let _res = p.oracle_try_update_prices(ProjectAccount::Scheduler, &to_prices_in(&[]))?;

    // assert_error(&res, ContractError::EmptyRawPriceItemVector);

    Ok(())
}

// the guard was removed
#[test]
fn zeros_in_prices() -> StdResult<()> {
    let mut p = Project::new();

    let _res = p.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Pinjeons, 0)]),
    )?;

    // assert_error(&res, ContractError::ZerosInPrices);

    Ok(())
}

#[test]
fn migrate_default() {
    let mut p = Project::new();

    p.app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            p.get_oracle_address(),
            &MigrateMsg {
                version: "2.0.0".to_string(),
            },
            p.get_oracle_code_id(),
        )
        .unwrap();
}
