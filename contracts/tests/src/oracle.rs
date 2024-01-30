use cosmwasm_std::StdResult;

use crate::helpers::{
    oracle::OracleExtension,
    suite::{core::Project, types::ProjectAccount},
};

#[test]
fn default() -> StdResult<()> {
    let mut project = Project::new();

    Ok(())
}
