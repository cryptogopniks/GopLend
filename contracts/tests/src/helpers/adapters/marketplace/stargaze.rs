use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::marketplace::stargaze::{
        msg::{ExecuteMsg, QueryMsg},
        types::Config,
    },
    error::parse_err,
};

use crate::helpers::suite::{
    core::Project,
    types::{ProjectAccount, ProjectNft},
};

pub trait AdapterMarketplaceStargazeExtension {
    fn adapter_marketplace_stargaze_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
        marketplace: &Option<Addr>,
        infinity_pool: &Option<Addr>,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_try_accept_bids(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config>;

    fn adapter_marketplace_stargaze_query_liquidation_bids_by_collection_address_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>;

    fn adapter_marketplace_stargaze_query_liquidation_bids_by_collection_address(
        &self,
        address: ProjectNft,
    ) -> StdResult<Vec<goplend_base::lending_platform::types::LiquidationBid>>;

    fn adapter_marketplace_stargaze_query_liquidation_bids_by_liquidator_address_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByLiquidatorAddressListResponseItem>>;

    fn adapter_marketplace_stargaze_query_liquidation_bids_by_liquidator_address(
        &self,
        address: ProjectAccount,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>;
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
        lending_platform: &Option<Addr>,
        marketplace: &Option<Addr>,
        infinity_pool: &Option<Addr>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_marketplace_stargaze_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
                    marketplace: marketplace.as_ref().map(|x| x.to_string()),
                    infinity_pool: infinity_pool.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_try_accept_bids(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_marketplace_stargaze_address(),
                &ExecuteMsg::AcceptBids {
                    bidded_collateral_item_list: bidded_collateral_item_list.to_owned(),
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
    fn adapter_marketplace_stargaze_query_liquidation_bids_by_collection_address_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>{
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryLiquidationBidsByCollectionAddressList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_liquidation_bids_by_collection_address(
        &self,
        address: ProjectNft,
    ) -> StdResult<Vec<goplend_base::lending_platform::types::LiquidationBid>> {
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryLiquidationBidsByCollectionAddress {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_liquidation_bids_by_liquidator_address_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByLiquidatorAddressListResponseItem>>{
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddressList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn adapter_marketplace_stargaze_query_liquidation_bids_by_liquidator_address(
        &self,
        address: ProjectAccount,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>{
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddress {
                address: address.to_string(),
            },
        )
    }
}
