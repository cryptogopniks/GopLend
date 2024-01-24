use cosmwasm_std::StdResult;
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    error::parse_err,
    lending_platform::{
        msg::{ExecuteMsg, QueryMsg},
        types::Config,
    },
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait LendingPlatformExtension {
    fn lending_platform_try_update_config<A: ToString>(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<A>,
        oracle: &Option<A>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_query_config(&self) -> StdResult<Config>;
}

impl LendingPlatformExtension for Project {
    #[track_caller]
    fn lending_platform_try_update_config<A: ToString>(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<A>,
        oracle: &Option<A>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::UpdateConfig {
                    owner: owner.as_ref().map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryConfig {},
        )
    }
}
