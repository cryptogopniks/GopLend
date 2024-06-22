use cosmwasm_std::{Addr, Decimal, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        marketplace::stargaze::{
            msg::{ExecuteMsg, QueryMsg},
            types::Config,
        },
    },
    assets::{Currency, TokenUnverified},
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
        oracle: &Option<Addr>,
        marketplace: &Option<Addr>,
        dex: &Option<Addr>,
        main_currency: &Option<Currency<TokenUnverified>>,
        price_impact: &Option<Decimal>,
        price_update_period: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_try_accept_offers(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
        route: &Route,
    ) -> StdResult<AppResponse>;

    fn adapter_marketplace_stargaze_query_config(&self) -> StdResult<Config>;

    fn adapter_marketplace_stargaze_query_offers_by_collection_address_list(
        &self,
        is_usd_price_required: bool,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
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
        oracle: &Option<Addr>,
        marketplace: &Option<Addr>,
        dex: &Option<Addr>,
        main_currency: &Option<Currency<TokenUnverified>>,
        price_impact: &Option<Decimal>,
        price_update_period: &Option<u64>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_adapter_marketplace_stargaze_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                    marketplace: marketplace.as_ref().map(|x| x.to_string()),
                    dex: dex.as_ref().map(|x| x.to_string()),
                    main_currency: main_currency.to_owned(),
                    price_impact: price_impact.to_owned(),
                    price_update_period: price_update_period.to_owned(),
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
    fn adapter_marketplace_stargaze_query_offers_by_collection_address_list(
        &self,
        is_usd_price_required: bool,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<goplend_base::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>>{
        self.app.wrap().query_wasm_smart(
            self.get_adapter_marketplace_stargaze_address(),
            &QueryMsg::QueryOffersByCollectionAddressList {
                is_usd_price_required,
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }
}
