use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        marketplace::stargaze::{
            msg::{ExecuteMsg, QueryCollectionOffersResponse, QueryMsg},
            types::Config,
        },
    },
    error::parse_err,
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait AdapterMarketplaceStargazeExtension {
    fn adapter_marketplace_stargaze_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn adapter_marketplace_stargaze_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        oracle: &Option<Addr>,
        marketplace: &Option<Addr>,
        adapter_dex: &Option<Addr>,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_try_accept_offers(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
        route: &Route,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_query_token_offers(
        &self,
        collection_address: impl ToString,
        token_list: &[&str],
        limit: u32,
    ) -> StdResult<Vec<(String, Vec<stargaze_marketplace_v2::orders::Bid>)>>;

    fn adapter_marketplace_stargaze_query_collection_offers(
        &self,
        collection_address: impl ToString,
        limit: u32,
    ) -> StdResult<QueryCollectionOffersResponse>;

    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config>;
}

impl AdapterMarketplaceStargazeExtension for Project {
    #[track_caller]
    fn adapter_marketplace_stargaze_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_marketplace_stargaze_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        oracle: &Option<Addr>,
        marketplace: &Option<Addr>,
        adapter_dex: &Option<Addr>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_marketplace_stargaze_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform_main: lending_platform_main.as_ref().map(|x| x.to_string()),
                    lending_platform_stable: lending_platform_stable
                        .as_ref()
                        .map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                    marketplace: marketplace.as_ref().map(|x| x.to_string()),
                    adapter_dex: adapter_dex.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_try_accept_offers(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
        route: &Route,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_marketplace_stargaze_address(),
                &ExecuteMsg::AcceptOffers {
                    bidded_collateral_item_list: bidded_collateral_item_list.to_owned(),
                    route: route.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_token_offers(
        &self,
        collection_address: impl ToString,
        token_list: &[&str],
        limit: u32,
    ) -> StdResult<Vec<(String, Vec<stargaze_marketplace_v2::orders::Bid>)>> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryTokenOffers {
                collection_address: collection_address.to_string(),
                token_list: token_list.iter().map(|x| x.to_string()).collect(),
                limit,
            },
        )
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_collection_offers(
        &self,
        collection_address: impl ToString,
        limit: u32,
    ) -> StdResult<QueryCollectionOffersResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryCollectionOffers {
                collection_address: collection_address.to_string(),
                limit,
            },
        )
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryConfig {},
        )
    }
}
