use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        scheduler::common::{
            msg::{ExecuteMsg, QueryMsg},
            types::{ClockSource, Config, Log},
        },
    },
    error::parse_err,
    lending_platform::types::LiquidationItem,
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait AdapterSchedulerCommonExtension {
    fn adapter_scheduler_common_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn adapter_scheduler_common_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        clock_source: &Option<ClockSource>,
        execution_cooldown: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn adapter_scheduler_common_try_push(
        &mut self,
        sender: ProjectAccount,
        clock_source: ClockSource,
        targets: &[LiquidationItem],
        route: &Route,
    ) -> StdResult<AppResponse>;

    fn adapter_scheduler_common_query_config(&self) -> StdResult<Config>;

    fn adapter_scheduler_common_query_log(&self) -> StdResult<Vec<Log>>;
}

impl AdapterSchedulerCommonExtension for Project {
    #[track_caller]
    fn adapter_scheduler_common_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_scheduler_common_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_scheduler_common_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        clock_source: &Option<ClockSource>,
        execution_cooldown: &Option<u64>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_scheduler_common_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform_main: lending_platform_main.as_ref().map(|x| x.to_string()),
                    lending_platform_stable: lending_platform_stable
                        .as_ref()
                        .map(|x| x.to_string()),
                    offchain_clock: offchain_clock
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect()),
                    clock_source: clock_source.to_owned(),
                    execution_cooldown: execution_cooldown.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_scheduler_common_try_push(
        &mut self,
        sender: ProjectAccount,
        clock_source: ClockSource,
        targets: &[LiquidationItem],
        route: &Route,
    ) -> StdResult<AppResponse> {
        match clock_source {
            ClockSource::Offchain => self
                .app
                .execute_contract(
                    sender.into(),
                    self.get_adapter_scheduler_common_address(),
                    &ExecuteMsg::Push {
                        targets: targets.to_owned(),
                        route: route.to_owned(),
                    },
                    &[],
                )
                .map_err(parse_err),
            ClockSource::Onchain => unimplemented!(),
        }
    }

    #[track_caller]
    fn adapter_scheduler_common_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_scheduler_common_address(),
            &QueryMsg::QueryConfig {},
        )
    }

    #[track_caller]
    fn adapter_scheduler_common_query_log(&self) -> StdResult<Vec<Log>> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_scheduler_common_address(),
            &QueryMsg::QueryLog {},
        )
    }
}
