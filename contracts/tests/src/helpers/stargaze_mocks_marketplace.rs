use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{AppResponse, Executor};

use stargaze_marketplace_v2::{
    msg::{ExecuteMsg, QueryMsg},
    orders::OrderDetails,
    state::Config,
};

use goplend_base::error::parse_err;
use stargaze_mocks_marketplace::{msg::SudoMsg, state::Bid};

use crate::helpers::suite::{core::Project, types::ProjectAccount};

pub trait StargazeMocksMarketplaceExtension {
    fn stargaze_mocks_marketplace_try_accept_collection_bid(
        &mut self,
        sender: ProjectAccount,
        id: &str,
        token_id: &str,
        details: OrderDetails<String>,
    ) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_try_accept_bid(
        &mut self,
        sender: ProjectAccount,
        id: &str,
        details: OrderDetails<String>,
    ) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_try_set_bids(&mut self, bids: &[Bid]) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_try_set_royalty_registry(
        &mut self,
        royalty_registry: &Addr,
    ) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_query_config(
        &self,
    ) -> StdResult<stargaze_marketplace_v2::state::Config<Addr>>;
}

impl StargazeMocksMarketplaceExtension for Project {
    #[track_caller]
    fn stargaze_mocks_marketplace_try_accept_collection_bid(
        &mut self,
        sender: ProjectAccount,
        id: &str,
        token_id: &str,
        details: OrderDetails<String>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_stargaze_mocks_marketplace_address(),
                &ExecuteMsg::AcceptCollectionBid {
                    id: id.to_string(),
                    token_id: token_id.to_string(),
                    details,
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn stargaze_mocks_marketplace_try_accept_bid(
        &mut self,
        sender: ProjectAccount,
        id: &str,
        details: OrderDetails<String>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_stargaze_mocks_marketplace_address(),
                &ExecuteMsg::AcceptBid {
                    id: id.to_string(),
                    details,
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn stargaze_mocks_marketplace_try_set_bids(&mut self, bids: &[Bid]) -> StdResult<AppResponse> {
        self.app
            .wasm_sudo(
                self.get_stargaze_mocks_marketplace_address(),
                &SudoMsg::SetBids {
                    bids: bids.to_owned(),
                },
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn stargaze_mocks_marketplace_try_set_royalty_registry(
        &mut self,
        royalty_registry: &Addr,
    ) -> StdResult<AppResponse> {
        self.app
            .wasm_sudo(
                self.get_stargaze_mocks_marketplace_address(),
                &SudoMsg::SetRoyaltyRegistry(royalty_registry.to_string()),
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn stargaze_mocks_marketplace_query_config(&self) -> StdResult<Config<Addr>> {
        self.app.wrap().query_wasm_smart(
            self.get_stargaze_mocks_marketplace_address(),
            &QueryMsg::Config {},
        )
    }
}
