use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    assets::TokenUnverified,
    error::parse_err,
    lending_platform::types::{BiddedCollateralItem, CollectionInfo},
    market_maker::{
        msg::{ExecuteMsg, QueryMsg},
        types::{CollectionOwner, Config, LiquidityInfo, OffersResponse},
    },
};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

use super::suite::types::{ProjectAsset, ProjectNft};

pub trait MarketMakerExtension {
    fn market_maker_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn market_maker_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        lending_platform: &Option<Addr>,
        oracle: &Option<Addr>,
        token: &Option<TokenUnverified>,
    ) -> StdResult<AppResponse>;

    fn market_maker_try_remove_offers(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        recipient: &Option<Addr>,
    ) -> StdResult<AppResponse>;

    fn market_maker_try_pause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn market_maker_try_unpause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn market_maker_try_set_collection(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        owner: ProjectAccount,
    ) -> StdResult<AppResponse>;

    fn market_maker_try_remove_collection(
        &mut self,
        sender: ProjectAccount,
        collection_list: &[ProjectNft],
    ) -> StdResult<AppResponse>;

    fn market_maker_try_deposit_liquidity(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn market_maker_try_withdraw_undistributed_liquidity(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
    ) -> StdResult<AppResponse>;

    fn market_maker_try_withdraw_collateral(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
    ) -> StdResult<AppResponse>;

    fn market_maker_try_update_offers(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        from_to_price_list: &[(u128, u128)],
    ) -> StdResult<AppResponse>;

    fn market_maker_try_accept_offers(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[BiddedCollateralItem],
    ) -> StdResult<AppResponse>;

    fn market_maker_query_config(&self) -> StdResult<Config>;

    fn market_maker_query_offer_prices_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<OffersResponse>>;

    fn market_maker_query_offer_prices(&self, collection: ProjectNft) -> StdResult<OffersResponse>;

    fn market_maker_query_collection_owner_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<CollectionOwner<Addr>>>;

    fn market_maker_query_collection_owner(
        &self,
        collection: ProjectNft,
    ) -> StdResult<CollectionOwner<Addr>>;

    fn market_maker_query_liquidity_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<LiquidityInfo>>;

    fn market_maker_query_liquidity(&self, collection: ProjectNft) -> StdResult<LiquidityInfo>;

    fn market_maker_query_collateral_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectAccount>,
    ) -> StdResult<Vec<CollectionInfo<Addr>>>;

    fn market_maker_query_collateral(
        &self,
        collection_owner: ProjectAccount,
    ) -> StdResult<CollectionInfo<Addr>>;
}

impl MarketMakerExtension for Project {
    #[track_caller]
    fn market_maker_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    // // any
    // AcceptAdminRole {},

    // // admin
    // UpdateConfig {
    //     admin: Option<String>,
    //     worker: Option<String>,
    //     controller: Option<Vec<String>>,
    //     lending_platform: Option<String>,
    //     oracle: Option<String>,
    //     token: Option<TokenUnverified>,
    // },

    // RemoveOffers {
    //     collection: String,
    //     recipient: Option<String>,
    // },

    // // admin, worker
    // Pause {},

    // Unpause {},

    // // collection owner
    // /// add new collection or rewrite existing
    // SetCollection {
    //     collection: String,
    //     owner: String,
    // },

    // RemoveCollections {
    //     collection_list: Vec<String>,
    // },

    // DepositLiquidity {
    //     collection: String,
    // },

    // WithdrawUndistributedLiquidity {
    //     collection: String,
    // },

    // /// withdraw all bought NFTs
    // WithdrawCollateral {
    //     collection: String,
    // },

    // // controller
    // UpdateOffers {
    //     collection: String,
    //     from_to_price_list: Vec<(Uint128, Uint128)>,
    // },

    // // lending platform
    // AcceptOffers {
    //     bidded_collateral_item_list: Vec<BiddedCollateralItem>,
    // },

    // #[returns(super::types::Config)]
    // QueryConfig {},

    // #[returns(Vec<super::types::OffersResponse>)]
    // QueryOfferPricesList {
    //     amount: u32,
    //     start_after: Option<String>,
    // },

    // #[returns(super::types::OffersResponse)]
    // QueryOfferPrices { collection: String },

    // #[returns(Vec<super::types::CollectionOwner<Addr>>)]
    // QueryCollectionOwnerList {
    //     amount: u32,
    //     start_after: Option<String>,
    // },

    // #[returns(super::types::CollectionOwner<Addr>)]
    // QueryCollectionOwner { collection: String },

    // #[returns(Vec<super::types::LiquidityInfo>)]
    // QueryLiquidityList {
    //     amount: u32,
    //     start_after: Option<String>,
    // },

    // #[returns(super::types::LiquidityInfo)]
    // QueryLiquidity { collection: String },

    // #[returns(Vec<crate::lending_platform::types::CollectionInfo<Addr>>)]
    // QueryCollateralList {
    //     amount: u32,
    //     start_after: Option<String>,
    // },

    // #[returns(crate::lending_platform::types::CollectionInfo<Addr>)]
    // QueryCollateral { collection: String },

    #[track_caller]
    fn market_maker_try_update_config(
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
                self.get_market_maker_address(),
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
    fn market_maker_try_accept_offers(
        &mut self,
        sender: ProjectAccount,
        bidded_collateral_item_list: &[goplend_base::lending_platform::types::BiddedCollateralItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::AcceptOffers {
                    bidded_collateral_item_list: bidded_collateral_item_list.to_owned(),
                    route: route.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_query_token_offers(
        &self,
        collection_address: impl ToString,
        token_list: &[&str],
        limit: u32,
    ) -> StdResult<Vec<(String, Vec<stargaze_marketplace_v2::orders::Bid>)>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryTokenOffers {
                collection_address: collection_address.to_string(),
                token_list: token_list.iter().map(|x| x.to_string()).collect(),
                limit,
            },
        )
    }

    #[track_caller]
    fn market_maker_query_collection_offers(
        &self,
        collection_address: impl ToString,
        limit: u32,
    ) -> StdResult<QueryCollectionOffersResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryCollectionOffers {
                collection_address: collection_address.to_string(),
                limit,
            },
        )
    }

    #[track_caller]
    fn market_maker_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_market_maker_address(), &QueryMsg::QueryConfig {})
    }
}
