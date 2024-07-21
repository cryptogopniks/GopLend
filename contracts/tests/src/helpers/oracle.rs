use cosmwasm_std::StdResult;
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    converters::str_to_dec,
    error::parse_err,
    oracle::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, Log, PriceItem, PriceRatioItem, RawPriceItem},
    },
};

use crate::helpers::suite::{
    core::Project,
    types::{ProjectAccount, ProjectNft},
};

pub trait OracleExtension {
    fn oracle_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn oracle_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        execution_cooldown: &Option<u64>,
        max_price_update_period: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn oracle_try_set_prices(
        &mut self,
        sender: ProjectAccount,
        price_ratio: &str,
        data: &[RawPriceItem],
    ) -> StdResult<AppResponse>;

    fn oracle_try_update_prices(
        &mut self,
        sender: ProjectAccount,
        price_ratio: &str,
        data: &[RawPriceItem],
    ) -> StdResult<AppResponse>;

    fn oracle_try_remove_prices(
        &mut self,
        sender: ProjectAccount,
        collections: &[impl ToString],
    ) -> StdResult<AppResponse>;

    fn oracle_query_config(&self) -> StdResult<Config>;

    fn oracle_query_prices(
        &self,
        collection_addresses: &Option<Vec<ProjectNft>>,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<PriceItem>>;

    fn oracle_query_price_ratio(&self) -> StdResult<PriceRatioItem>;

    fn oracle_query_block_time(&self) -> StdResult<u64>;

    fn oracle_query_log(&self) -> StdResult<Vec<Log>>;
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
    fn oracle_try_set_prices(
        &mut self,
        sender: ProjectAccount,
        price_ratio: &str,
        data: &[RawPriceItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::SetPrices {
                    price_ratio: str_to_dec(price_ratio),
                    data: data.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn oracle_try_update_prices(
        &mut self,
        sender: ProjectAccount,
        price_ratio: &str,
        data: &[RawPriceItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_oracle_address(),
                &ExecuteMsg::UpdatePrices {
                    price_ratio: str_to_dec(price_ratio),
                    data: data.to_owned(),
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
        collection_addresses: &Option<Vec<ProjectNft>>,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<PriceItem>> {
        let collection_addresses = collection_addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_oracle_address(),
            &QueryMsg::QueryPrices {
                collection_addresses,
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn oracle_query_price_ratio(&self) -> StdResult<PriceRatioItem> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_oracle_address(), &QueryMsg::QueryPriceRatio {})
    }

    #[track_caller]
    fn oracle_query_block_time(&self) -> StdResult<u64> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_oracle_address(), &QueryMsg::QueryBlockTime {})
    }

    #[track_caller]
    fn oracle_query_log(&self) -> StdResult<Vec<Log>> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_oracle_address(), &QueryMsg::QueryLog {})
    }
}
