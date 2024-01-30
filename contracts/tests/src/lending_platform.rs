use cosmwasm_std::StdResult;

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    suite::{core::Project, types::ProjectAccount},
};

#[test]
fn create_bgl_token_default() -> StdResult<()> {
    let mut project = Project::new();

    let res = project.lending_platform_query_config()?;

    Ok(())
}
