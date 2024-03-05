use cosmwasm_std::StdResult;
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    error::parse_err,
    oracle::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, PriceItem, RawPriceItem},
    },
};

use crate::helpers::suite::{
    core::Project,
    types::{ProjectAccount, ProjectNft},
};

pub trait OracleExtension {
    fn oracle_try_update_config(
        &mut self,
        sender: ProjectAccount,
        scheduler: &Option<ProjectAccount>,
    ) -> StdResult<AppResponse>;

    fn oracle_try_update_prices(
        &mut self,
        sender: ProjectAccount,
        data: &[RawPriceItem],
    ) -> StdResult<AppResponse>;

    fn oracle_query_config(&self) -> StdResult<Config>;

    fn oracle_query_prices(
        &self,
        collection_addresses: &Option<Vec<ProjectNft>>,
    ) -> StdResult<Vec<PriceItem>>;

    fn oracle_query_block_time(&self) -> StdResult<u64>;
}

impl OracleExtension for Project {
    #[track_caller]
    fn oracle_try_update_config(
        &mut self,
        sender: ProjectAccount,
        scheduler: &Option<ProjectAccount>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::UpdateConfig {
                    scheduler: scheduler.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn oracle_try_update_prices(
        &mut self,
        sender: ProjectAccount,
        data: &[RawPriceItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::UpdatePrices {
                    data: data.to_owned(),
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
        collection_addresses: &Option<Vec<ProjectNft>>,
    ) -> StdResult<Vec<PriceItem>> {
        let collection_addresses = collection_addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_oracle_address(),
            &QueryMsg::QueryPrices {
                collection_addresses,
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
