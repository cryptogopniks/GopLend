use cosmwasm_std::{StdResult, Uint128};
use cw_multi_test::{AppResponse, Executor};

use sg_marketplace::msg::{
    CollectionBidOffset, CollectionBidsResponse, CollectionOffset, ExecuteMsg, QueryMsg,
};

use goplend_base::error::parse_err;
use stargaze_mocks_marketplace::{msg::SudoMsg, state::Bid};

use crate::helpers::suite::{
    core::Project,
    types::{ProjectAccount, ProjectNft},
};

pub trait StargazeMocksMarketplaceExtension {
    fn stargaze_mocks_marketplace_try_accept_collection_bid(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        token_id: &str,
        bidder: ProjectAccount,
        finder: &Option<&str>,
    ) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_try_accept_bid(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        token_id: &str,
        bidder: ProjectAccount,
        finder: &Option<&str>,
    ) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_try_set_bids(&mut self, bids: &[Bid]) -> StdResult<AppResponse>;

    fn stargaze_mocks_marketplace_collection_bids_sorted_by_price(
        &self,
        collection: ProjectNft,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<CollectionBidsResponse>;

    fn stargaze_mocks_marketplace_collection_collection_bids_by_bidder(
        &self,
        bidder: ProjectAccount,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<CollectionBidsResponse>;
}

impl StargazeMocksMarketplaceExtension for Project {
    #[track_caller]
    fn stargaze_mocks_marketplace_try_accept_collection_bid(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        token_id: &str,
        bidder: ProjectAccount,
        finder: &Option<&str>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_stargaze_mocks_marketplace_address(),
                &ExecuteMsg::AcceptCollectionBid {
                    collection: collection.to_string(),
                    token_id: token_id.parse().unwrap(),
                    bidder: bidder.to_string(),
                    finder: finder.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn stargaze_mocks_marketplace_try_accept_bid(
        &mut self,
        sender: ProjectAccount,
        collection: ProjectNft,
        token_id: &str,
        bidder: ProjectAccount,
        finder: &Option<&str>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_stargaze_mocks_marketplace_address(),
                &ExecuteMsg::AcceptBid {
                    collection: collection.to_string(),
                    token_id: token_id.parse().unwrap(),
                    bidder: bidder.to_string(),
                    finder: finder.as_ref().map(|x| x.to_string()),
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
    fn stargaze_mocks_marketplace_collection_bids_sorted_by_price(
        &self,
        collection: ProjectNft,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<CollectionBidsResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_stargaze_mocks_marketplace_address(),
            &QueryMsg::CollectionBidsSortedByPrice {
                collection: collection.to_string(),
                start_after: start_after.as_ref().map(|x| CollectionBidOffset {
                    collection: x.to_string(),
                    bidder: String::default(),
                    price: Uint128::default(),
                }),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn stargaze_mocks_marketplace_collection_collection_bids_by_bidder(
        &self,
        bidder: ProjectAccount,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<CollectionBidsResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_stargaze_mocks_marketplace_address(),
            &QueryMsg::CollectionBidsByBidder {
                bidder: bidder.to_string(),
                start_after: start_after.as_ref().map(|x| CollectionOffset {
                    collection: x.to_string(),
                    token_id: u32::default(),
                }),
                limit: limit.to_owned(),
            },
        )
    }
}
