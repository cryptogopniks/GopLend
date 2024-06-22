use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::dex::stargaze::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, Route},
    },
    converters::str_to_dec,
    error::parse_err,
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait AdapterDexStargazeMocksExtension {
    fn adapter_dex_stargaze_mocks_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse>;

    fn adapter_dex_stargaze_mocks_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,
        recover_address: &Option<Addr>,
        max_price_impact: &Option<&str>,
    ) -> StdResult<AppResponse>;

    fn adapter_dex_stargaze_mocks_try_swap(
        &mut self,
        sender: ProjectAccount,
        route: &Route,
        price_ratio: &str,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config>;
}

impl AdapterDexStargazeMocksExtension for Project {
    #[track_caller]
    fn adapter_dex_stargaze_mocks_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_dex_stargaze_mocks_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_dex_stargaze_mocks_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,
        recover_address: &Option<Addr>,
        max_price_impact: &Option<&str>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_dex_stargaze_mocks_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
                    adapter_marketplace: adapter_marketplace.as_ref().map(|x| x.to_string()),
                    recover_address: recover_address.as_ref().map(|x| x.to_string()),
                    max_price_impact: max_price_impact.as_ref().map(|x| str_to_dec(x)),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_dex_stargaze_mocks_try_swap(
        &mut self,
        sender: ProjectAccount,
        route: &Route,
        price_ratio: &str,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_dex_stargaze_mocks_address(),
                &ExecuteMsg::Swap {
                    route: route.to_owned(),
                    price_ratio: str_to_dec(price_ratio),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_dex_stargaze_mocks_address(),
            &QueryMsg::QueryConfig {},
        )
    }
}
