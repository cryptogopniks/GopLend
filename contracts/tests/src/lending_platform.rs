use speculoos::assert_that;

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
