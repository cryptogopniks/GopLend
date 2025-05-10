use cw_multi_test::Executor;
use pretty_assertions::assert_eq;

use cosmwasm_std::StdResult;

use goplend_base::{
    converters::{str_to_dec, u128_to_dec},
    error::ContractError,
    oracle::{
        msg::MigrateMsg,
        types::{split_address, PriceItem, QueryPricesResponse},
    },
};

use crate::helpers::{
    oracle::OracleExtension,
    suite::{
        core::{assert_error, Project},
        types::{ProjectAccount, ProjectNft},
    },
};

pub const DECIMALS_MULTIPLIER: u128 = 1_000_000;

pub fn to_prices_in(list: &[(ProjectNft, &str)]) -> Vec<(String, u32)> {
    const USD_DECIMALS: u32 = 2;

    list.iter()
        .map(|(collection, price)| {
            let (_, collection) = split_address(collection);
            let price = TryInto::<u32>::try_into(
                (str_to_dec(&price.replace('_', "")) * (u128_to_dec(10u128)).pow(USD_DECIMALS))
                    .to_uint_floor()
                    .u128(),
            )
            .unwrap();

            (collection, price)
        })
        .collect()
}

pub fn to_prices_out(list: &[(ProjectNft, &str)]) -> Vec<PriceItem> {
    const USDC_DECIMALS: u32 = 6;

    let mut prices: Vec<PriceItem> = list
        .iter()
        .map(|(collection, price)| {
            let price = (str_to_dec(price) * (u128_to_dec(10u128)).pow(USDC_DECIMALS))
                .to_uint_floor()
                .u128();

            PriceItem::new(collection, price)
        })
        .collect();
    prices.sort_unstable_by_key(|x| x.collection.clone());

    prices
}

#[test]
fn migrate_default() {
    let mut p = Project::new();

    p.app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            p.get_oracle_address(),
            &MigrateMsg {
                version: "2.0.1".to_string(),
            },
            p.get_oracle_code_id(),
        )
        .unwrap();
}

#[test]
fn default() -> StdResult<()> {
    let mut p = Project::new();

    p.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[
            (ProjectNft::Gopniks, "800.00"),
            (ProjectNft::Pinjeons, "300.00"),
        ]),
    )?;

    let res = p.oracle_query_prices(9, &None::<ProjectNft>, &None::<Vec<ProjectNft>>)?;

    p.wait(3);

    assert_eq!(
        res.data,
        to_prices_out(&[
            (ProjectNft::Gopniks, "800.00"),
            (ProjectNft::Pinjeons, "300.00"),
        ])
    );

    Ok(())
}

#[test]
fn unauthorized() -> StdResult<()> {
    let mut p = Project::new();

    let res = p
        .oracle_try_update_prices(
            ProjectAccount::Admin,
            &to_prices_in(&[
                (ProjectNft::Gopniks, "800.00"),
                (ProjectNft::Pinjeons, "300.00"),
            ]),
        )
        .unwrap_err();

    assert_error(res, ContractError::Unauthorized);

    Ok(())
}

// the guard was removed
#[test]
fn duplicated_collections() -> StdResult<()> {
    let mut p = Project::new();

    let _res = p.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[
            (ProjectNft::Pinjeons, "300.00"),
            (ProjectNft::Pinjeons, "300.00"),
        ]),
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
        &to_prices_in(&[(ProjectNft::Pinjeons, "0.00")]),
    )?;

    // assert_error(&res, ContractError::ZerosInPrices);

    Ok(())
}

#[test]
fn query_prices_pagination() -> StdResult<()> {
    let mut p = Project::new();

    p.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[
            (ProjectNft::Gopniks, "800.00"),
            (ProjectNft::Pinjeons, "300.00"),
        ]),
    )?;

    let QueryPricesResponse {
        data: data_full, ..
    } = p.oracle_query_prices(9, &None::<ProjectNft>, &None::<Vec<ProjectNft>>)?;
    assert_eq!(data_full.len(), 2);

    let QueryPricesResponse { data, .. } = p.oracle_query_prices(
        9,
        &Some(data_full.first().unwrap().collection.to_owned()),
        &None::<Vec<ProjectNft>>,
    )?;
    assert_eq!(data.len(), 1);

    let QueryPricesResponse { data, .. } = p.oracle_query_prices(
        9,
        &Some(data_full.last().unwrap().collection.to_owned()),
        &None::<Vec<ProjectNft>>,
    )?;
    assert_eq!(data.len(), 0);

    Ok(())
}
