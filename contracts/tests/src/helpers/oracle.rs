use cosmwasm_std::StdResult;
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    error::parse_err,
    oracle::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, QueryPricesResponse},
    },
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait OracleExtension {
    fn oracle_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn oracle_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        execution_cooldown: &Option<u64>,
        max_price_update_period: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn oracle_try_update_prices(
        &mut self,
        sender: ProjectAccount,
        data: &[(impl ToString, u32)],
    ) -> StdResult<AppResponse>;

    fn oracle_try_remove_prices(
        &mut self,
        sender: ProjectAccount,
        collections: &[impl ToString],
    ) -> StdResult<AppResponse>;

    fn oracle_query_config(&self) -> StdResult<Config>;

    fn oracle_query_prices(
        &self,
        amount: u32,
        start_after: &Option<impl ToString>,
        collections: &Option<Vec<impl ToString>>,
    ) -> StdResult<QueryPricesResponse>;

    fn oracle_query_block_time(&self) -> StdResult<u64>;
}

impl OracleExtension for Project {
    #[track_caller]
    fn oracle_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn oracle_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        execution_cooldown: &Option<u64>,
        max_price_update_period: &Option<u64>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    controller: controller
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect()),
                    execution_cooldown: execution_cooldown.to_owned(),
                    max_price_update_period: max_price_update_period.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn oracle_try_update_prices(
        &mut self,
        sender: ProjectAccount,
        data: &[(impl ToString, u32)],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::UpdatePrices {
                    data: data
                        .iter()
                        .map(|(postfix, price)| (postfix.to_string(), *price))
                        .collect(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn oracle_try_remove_prices(
        &mut self,
        sender: ProjectAccount,
        collections: &[impl ToString],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::RemovePrices {
                    collections: collections.iter().map(|x| x.to_string()).collect(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn oracle_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_oracle_address(), &QueryMsg::QueryConfig {})
    }

    #[track_caller]
    fn oracle_query_prices(
        &self,
        amount: u32,
        start_after: &Option<impl ToString>,
        collections: &Option<Vec<impl ToString>>,
    ) -> StdResult<QueryPricesResponse> {
        let collections = collections
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_oracle_address(),
            &QueryMsg::QueryPrices {
                collections,
                start_after: start_after.as_ref().map(|x| x.to_string()),
                amount,
            },
        )
    }

    #[track_caller]
    fn oracle_query_block_time(&self) -> StdResult<u64> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_oracle_address(), &QueryMsg::QueryBlockTime {})
    }
}
