use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::scheduler::kujira::{
        msg::{ExecuteMsg, QueryMsg},
        types::{ClockSource, Config},
    },
    error::parse_err,
    lending_platform::types::LiquidationItem,
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait AdapterSchedulerKujiraExtension {
    fn adapter_scheduler_kujira_try_update_config(
        &mut self,
        sender: ProjectAccount,
        lending_platform: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        clock_source: &Option<ClockSource>,
        min_clock_period: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn adapter_scheduler_kujira_try_push(
        &mut self,
        sender: ProjectAccount,
        clock_source: ClockSource,
        targets: &Vec<LiquidationItem>,
    ) -> StdResult<AppResponse>;

    fn adapter_scheduler_kujira_query_config(&self) -> StdResult<Config>;

    fn adapter_scheduler_kujira_query_log(&self, amount: u32) -> StdResult<Vec<(u64, u32)>>;
}

impl AdapterSchedulerKujiraExtension for Project {
    #[track_caller]
    fn adapter_scheduler_kujira_try_update_config(
        &mut self,
        sender: ProjectAccount,
        lending_platform: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        clock_source: &Option<ClockSource>,
        min_clock_period: &Option<u64>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_scheduler_kujira_address(),
                &ExecuteMsg::UpdateConfig {
                    lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
                    offchain_clock: offchain_clock
                        .as_ref()
                        .map(|x| x.into_iter().map(|y| y.to_string()).collect()),
                    clock_source: clock_source.to_owned(),
                    min_clock_period: min_clock_period.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_scheduler_kujira_try_push(
        &mut self,
        sender: ProjectAccount,
        clock_source: ClockSource,
        targets: &Vec<LiquidationItem>,
    ) -> StdResult<AppResponse> {
        match clock_source {
            ClockSource::Offchain => self
                .app
                .execute_contract(
                    sender.into(),
                    self.get_adapter_scheduler_kujira_address(),
                    &ExecuteMsg::Push {
                        targets: targets.to_owned(),
                    },
                    &[],
                )
                .map_err(parse_err),
            ClockSource::Onchain => unimplemented!(),
        }
    }

    #[track_caller]
    fn adapter_scheduler_kujira_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_scheduler_kujira_address(),
            &QueryMsg::QueryConfig {},
        )
    }

    #[track_caller]
    fn adapter_scheduler_kujira_query_log(&self, amount: u32) -> StdResult<Vec<(u64, u32)>> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_scheduler_kujira_address(),
            &QueryMsg::QueryLog { amount },
        )
    }
}
