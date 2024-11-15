use cosmwasm_std::{Addr, StdResult, Uint128};
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

use crate::helpers::suite::{
    core::{add_funds_to_exec_msg, Project},
    types::{ProjectAccount, ProjectAsset, ProjectNft},
};

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

    fn market_maker_query_offer_prices(
        &self,
        collection: impl ToString,
    ) -> StdResult<OffersResponse>;

    fn market_maker_query_collection_owner_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<CollectionOwner<Addr>>>;

    fn market_maker_query_collection_owner(
        &self,
        collection: impl ToString,
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

    #[track_caller]
    fn market_maker_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        lending_platform: &Option<Addr>,
        oracle: &Option<Addr>,
        token: &Option<TokenUnverified>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    controller: controller
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect()),
                    lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                    token: token.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_remove_offers(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        recipient: &Option<Addr>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::RemoveOffers {
                    collection: collection.to_string(),
                    recipient: recipient.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_pause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::Pause {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_unpause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::Unpause {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_set_collection(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        owner: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::SetCollection {
                    collection: collection.to_string(),
                    owner: owner.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_remove_collection(
        &mut self,
        sender: ProjectAccount,
        collection_list: &[ProjectNft],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::RemoveCollections {
                    collection_list: collection_list.iter().map(|x| x.to_string()).collect(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_deposit_liquidity(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        add_funds_to_exec_msg(
            self,
            sender,
            &self.get_lending_platform_address(),
            &ExecuteMsg::DepositLiquidity {
                collection: collection.to_string(),
            },
            amount,
            asset,
        )
    }

    #[track_caller]
    fn market_maker_try_withdraw_undistributed_liquidity(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::WithdrawUndistributedLiquidity {
                    collection: collection.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_withdraw_collateral(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::WithdrawCollateral {
                    collection: collection.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_try_update_offers(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        from_to_price_list: &[(u128, u128)],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_market_maker_address(),
                &ExecuteMsg::UpdateOffers {
                    collection: collection.to_string(),
                    from_to_price_list: from_to_price_list
                        .iter()
                        .map(|(from, to)| (Uint128::new(*from), Uint128::new(*to)))
                        .collect(),
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
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn market_maker_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_market_maker_address(), &QueryMsg::QueryConfig {})
    }

    #[track_caller]
    fn market_maker_query_offer_prices_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<OffersResponse>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryOfferPricesList {
                amount,
                start_after: start_after.map(|x| x.to_string()),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_offer_prices(
        &self,
        collection: impl ToString,
    ) -> StdResult<OffersResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryOfferPrices {
                collection: collection.to_string(),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_collection_owner_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<CollectionOwner<Addr>>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryCollectionOwnerList {
                amount,
                start_after: start_after.map(|x| x.to_string()),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_collection_owner(
        &self,
        collection: impl ToString,
    ) -> StdResult<CollectionOwner<Addr>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryCollectionOwner {
                collection: collection.to_string(),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_liquidity_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectNft>,
    ) -> StdResult<Vec<LiquidityInfo>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryLiquidityList {
                amount,
                start_after: start_after.map(|x| x.to_string()),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_liquidity(&self, collection: ProjectNft) -> StdResult<LiquidityInfo> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryLiquidity {
                collection: collection.to_string(),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_collateral_list(
        &self,
        amount: u32,
        start_after: &Option<ProjectAccount>,
    ) -> StdResult<Vec<CollectionInfo<Addr>>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryCollateralList {
                amount,
                start_after: start_after.map(|x| x.to_string()),
            },
        )
    }

    #[track_caller]
    fn market_maker_query_collateral(
        &self,
        collection_owner: ProjectAccount,
    ) -> StdResult<CollectionInfo<Addr>> {
        self.app.wrap().query_wasm_smart(
            self.get_market_maker_address(),
            &QueryMsg::QueryCollateral {
                collection_owner: collection_owner.to_string(),
            },
        )
    }
}
