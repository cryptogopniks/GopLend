use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        marketplace::stargaze::{
            msg::{ExecuteMsg, QueryMsg, QueryOffersByCollectionAddressResponse},
            types::{Config, MarketplaceVersion, PlatformType},
        },
    },
    error::parse_err,
};
use sg_marketplace::msg::{BidOffset, CollectionBidOffset};

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
        marketplace_version: &Option<MarketplaceVersion>,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_try_accept_offers(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
        route: &Route,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config>;

    fn adapter_marketplace_stargaze_query_offers_by_collection_address(
        &self,
        platform_type: PlatformType,
        collection_address: impl ToString,
        expiration_delay: u64,
        start_after_collection_offer: &Option<CollectionBidOffset>,
        start_after_token_offer: &Option<BidOffset>,
        limit: &Option<u32>,
    ) -> StdResult<QueryOffersByCollectionAddressResponse>;
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
        marketplace_version: &Option<MarketplaceVersion>,
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
                    marketplace_version: marketplace_version.to_owned(),
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
    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryConfig {},
        )
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_offers_by_collection_address(
        &self,
        platform_type: PlatformType,
        collection_address: impl ToString,
        expiration_delay: u64,
        start_after_collection_offer: &Option<CollectionBidOffset>,
        start_after_token_offer: &Option<BidOffset>,
        limit: &Option<u32>,
    ) -> StdResult<QueryOffersByCollectionAddressResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryOffersByCollectionAddress {
                platform_type,
                collection_address: collection_address.to_string(),
                expiration_delay,
                start_after_collection_offer: start_after_collection_offer.to_owned(),
                start_after_token_offer: start_after_token_offer.to_owned(),
                limit: limit.to_owned(),
            },
        )
    }
}
