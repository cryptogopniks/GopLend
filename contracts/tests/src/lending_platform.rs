use cosmwasm_std::StdResult;

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    suite::{core::Project, types::ProjectAccount},
};

#[test]
fn default() -> StdResult<()> {
    let mut project = Project::new();

    project.lending_platform_try_update_config(
        ProjectAccount::Admin,
        &Some(ProjectAccount::Alice),
        &Some(ProjectAccount::Bob),
    )?;

    Ok(())
}
