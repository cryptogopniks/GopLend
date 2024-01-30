use cosmwasm_std::StdResult;

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    suite::{core::Project, types::ProjectAccount},
};

#[test]
fn default() -> StdResult<()> {
    let mut project = Project::new();

    Ok(())
}
