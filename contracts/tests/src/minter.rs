use speculoos::assert_that;

use cosmwasm_std::{Addr, StdResult};

use goplend_base::{
    assets::{Currency, Token},
    error::ContractError,
    minter::types::CurrencyInfo,
};

use crate::helpers::{
    minter::MinterExtension,
    suite::{
        core::{assert_error, Project},
        types::ProjectAccount,
    },
};

const ADDRESS_A: &str = "wasm1nwnejwsdpqktusvh8qhxe5arsznjd5asdwutmaz9n5qcpl3dcmhsaa42gt";

#[test]
fn update_token_owner() -> StdResult<()> {
    let mut project = Project::new();

    let currencies = project.minter_query_currency_info_list(10, &None)?;
    assert_that(&currencies).is_equal_to(vec![CurrencyInfo {
        currency: Currency::new(&Token::new_cw20(&Addr::unchecked(ADDRESS_A)), 6),
        cw20_code_id: Some(1),
        owner: ProjectAccount::Admin.into(),
        whitelist: vec![project.get_lending_platform_address()],
        permissionless_burning: false,
    }]);

    project.minter_try_update_currency_info(
        ProjectAccount::Admin,
        &currencies[0].currency.token.get_denom_or_address(),
        &Some(ProjectAccount::Alice),
        &None,
        &None,
    )?;

    let res = project
        .minter_try_accept_token_owner_role(ProjectAccount::Bob)
        .unwrap_err();
    assert_error(&res, ContractError::Unauthorized);

    project.minter_try_accept_token_owner_role(ProjectAccount::Alice)?;

    let currencies = project.minter_query_currency_info_list(10, &None)?;
    assert_that(&currencies).is_equal_to(vec![CurrencyInfo {
        currency: Currency::new(&Token::new_cw20(&Addr::unchecked(ADDRESS_A)), 6),
        cw20_code_id: Some(1),
        owner: ProjectAccount::Alice.into(),
        whitelist: vec![project.get_lending_platform_address()],
        permissionless_burning: false,
    }]);

    Ok(())
}
