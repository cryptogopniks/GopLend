use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    error::parse_err,
    lending_platform::types::LiquidationItem,
    scheduler::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, Log},
    },
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait SchedulerExtension {
    fn scheduler_try_accept_admin_role(&mut self, sender: ProjectAccount)
        -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn scheduler_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        execution_cooldown: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn scheduler_try_push(
        &mut self,
        sender: ProjectAccount,
        targets: &[LiquidationItem],
    ) -> StdResult<AppResponse>;

    fn scheduler_query_config(&self) -> StdResult<Config>;

    fn scheduler_query_log(&self) -> StdResult<Vec<Log>>;
}

impl SchedulerExtension for Project {
    #[track_caller]
    fn scheduler_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_scheduler_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn scheduler_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        execution_cooldown: &Option<u64>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_scheduler_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
                    offchain_clock: offchain_clock
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect()),
                    execution_cooldown: execution_cooldown.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn scheduler_try_push(
        &mut self,
        sender: ProjectAccount,
        targets: &[LiquidationItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_scheduler_address(),
                &ExecuteMsg::Push {
                    targets: targets.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn scheduler_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_scheduler_address(), &QueryMsg::QueryConfig {})
    }

    #[track_caller]
    fn scheduler_query_log(&self) -> StdResult<Vec<Log>> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_scheduler_address(), &QueryMsg::QueryLog {})
    }
}
