use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::str_to_dec,
    error::parse_err,
    lending_platform::{
        msg::{
            ExecuteMsg, QueryAmountsResponse, QueryBorrowersResponseItem,
            QueryCollateralsResponseItem, QueryCollectionsResponseItem,
            QueryLiquidationBidsByCollectionAddressListResponseItem,
            QueryLiquidationBidsByLiquidatorAddressListResponseItem, QueryLiquidatorsResponseItem,
            QueryMsg, QueryProposalsResponseItem, QueryUnbondersResponseItem,
        },
        types::{
            AddressConfig, Balances, Borrower, Collateral, Collection, CollectionInfo,
            CommonConfig, LiquidationBid, LiquidationItem, Liquidator, Proposal, RateConfig,
            Unbonder,
        },
    },
};

use crate::helpers::suite::{
    core::{add_funds_to_exec_msg, add_token_to_exec_msg, Project},
    types::{ProjectAccount, ProjectAsset, ProjectNft},
};

pub trait LendingPlatformExtension {
    fn lending_platform_try_deposit(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_unbond(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: &Token,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_withdraw(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn lending_platform_try_deposit_collateral(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_withdraw_collateral(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_borrow(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_repay(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_place_bid(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
        discount: &str,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_remove_bid(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
        creation_date: u64,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_update_bid(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
        creation_date: u64,
        amount: u128,
        discount: &str,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_liquidate(
        &mut self,
        sender: &Addr,
        targets: &[LiquidationItem],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_update_address_config(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        oracle: &Option<Addr>,
        minter: &Option<Addr>,
        scheduler: &Option<Addr>,
        marketplace: &Option<Addr>,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn lending_platform_try_update_rate_config(
        &mut self,
        sender: ProjectAccount,
        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn lending_platform_try_update_common_config(
        &mut self,
        sender: ProjectAccount,
        main_currency: &Option<Currency<TokenUnverified>>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        unbonding_period: &Option<u64>,
        collateral_min_value: &Option<u128>,
        min_available_liquidity_multiplier: &Option<&str>,
        price_update_period: &Option<u64>,
        is_marketplace_interface_enabled: &Option<bool>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_deposit_reserve_liquidity(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_lock(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn lending_platform_try_unlock(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn lending_platform_try_distribute_funds(
        &mut self,
        sender: ProjectAccount,
        address_and_weight_list: &[(impl ToString, &str)],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_remove_collection(
        &mut self,
        sender: ProjectAccount,
        address: ProjectNft,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_create_proposal(
        &mut self,
        sender: ProjectAccount,
        proposal: &Proposal<String, TokenUnverified>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_reject_proposal(
        &mut self,
        sender: ProjectAccount,
        id: u32,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_accept_proposal(
        &mut self,
        sender: ProjectAccount,
        id: u32,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_query_address_config(&self) -> StdResult<AddressConfig>;

    fn lending_platform_query_rate_config(&self) -> StdResult<RateConfig>;

    fn lending_platform_query_common_config(&self) -> StdResult<CommonConfig>;

    fn lending_platform_query_platform_revenue(&self) -> StdResult<Vec<Funds<Token>>>;

    fn lending_platform_query_balances(&self) -> StdResult<Balances>;

    fn lending_platform_query_unbonder_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryUnbondersResponseItem>>;

    fn lending_platform_query_unbonder(&self, address: ProjectAccount) -> StdResult<Unbonder>;

    fn lending_platform_query_borrower_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryBorrowersResponseItem>>;

    fn lending_platform_query_borrower(&self, address: ProjectAccount) -> StdResult<Borrower>;

    fn lending_platform_query_liquidator_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidatorsResponseItem>>;

    fn lending_platform_query_liquidator(&self, address: ProjectAccount) -> StdResult<Liquidator>;

    fn lending_platform_query_collateral_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>>;

    fn lending_platform_query_collateral(&self, address: ProjectNft) -> StdResult<Collateral>;

    fn lending_platform_query_liquidation_bids_by_collection_address_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_collection_address(
        &self,
        address: ProjectNft,
    ) -> StdResult<Vec<LiquidationBid>>;

    fn lending_platform_query_liquidation_bids_by_liquidator_address_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_liquidator_address(
        &self,
        address: ProjectAccount,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>;

    fn lending_platform_query_proposals(
        &self,
        start_after: &Option<u32>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryProposalsResponseItem>>;

    fn lending_platform_query_collection_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollectionsResponseItem>>;

    fn lending_platform_query_collection(&self, address: ProjectNft)
        -> StdResult<Collection<Addr>>;

    fn lending_platform_query_bgl_currency_to_main_currency_price(&self) -> StdResult<Decimal>;

    fn lending_platform_query_conditional_deposit_apr(
        &self,
        amount_to_deposit: &Option<u128>,
    ) -> StdResult<Decimal>;

    fn lending_platform_query_ltv_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<(Addr, Decimal)>>;

    fn lending_platform_query_conditional_ltv(
        &self,
        borrower: ProjectAccount,
        amount_to_deposit: &Option<u128>,
        amount_to_borrow: &Option<u128>,
    ) -> StdResult<Decimal>;

    fn lending_platform_query_total_available_to_borrow_liquidity(&self) -> StdResult<Uint128>;

    fn lending_platform_query_available_to_borrow(
        &self,
        borrower: ProjectAccount,
        target_ltv: &Option<&str>,
    ) -> StdResult<Uint128>;

    fn lending_platform_query_amounts(&self) -> StdResult<QueryAmountsResponse>;

    fn lending_platform_query_targets(
        &self,
        borrowers: &[impl ToString],
    ) -> StdResult<Vec<LiquidationItem>>;
}

impl LendingPlatformExtension for Project {
    #[track_caller]
    fn lending_platform_try_deposit(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::Deposit {};

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_unbond(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: &Token,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::Unbond {};

        add_token_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_withdraw(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::Withdraw {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_deposit_collateral(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::DepositCollateral {
                    collections: collections.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_withdraw_collateral(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::WithdrawCollateral {
                    collections: collections.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_borrow(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::Borrow {
                    amount: Uint128::from(amount),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_repay(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::Repay {};

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_place_bid(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
        discount: &str,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::PlaceBid {
            collections: collections.to_owned(),
            discount: str_to_dec(discount),
        };

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_remove_bid(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
        creation_date: u64,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::RemoveBid {
                    collections: collections.to_owned(),
                    creation_date,
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_bid(
        &mut self,
        sender: ProjectAccount,
        collections: &[CollectionInfo<String>],
        creation_date: u64,
        amount: u128,
        discount: &str,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::UpdateBid {
            collections: collections.to_owned(),
            creation_date,
            amount: Uint128::from(amount),
            discount: str_to_dec(discount),
        };

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_liquidate(
        &mut self,
        sender: &Addr,
        targets: &[LiquidationItem],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.to_owned(),
                self.get_lending_platform_address(),
                &ExecuteMsg::Liquidate {
                    targets: targets.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_address_config(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        oracle: &Option<Addr>,
        minter: &Option<Addr>,
        scheduler: &Option<Addr>,
        marketplace: &Option<Addr>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::UpdateAddressConfig {
                    owner: owner.as_ref().map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                    minter: minter.as_ref().map(|x| x.to_string()),
                    scheduler: scheduler.as_ref().map(|x| x.to_string()),
                    marketplace: marketplace.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_rate_config(
        &mut self,
        sender: ProjectAccount,
        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::UpdateRateConfig {
                    borrow_apr: borrow_apr.as_ref().map(|x| str_to_dec(x)),
                    borrow_fee_rate: borrow_fee_rate.as_ref().map(|x| str_to_dec(x)),
                    liquidation_fee_rate: liquidation_fee_rate.as_ref().map(|x| str_to_dec(x)),
                    discount_min_rate: discount_min_rate.as_ref().map(|x| str_to_dec(x)),
                    discount_max_rate: discount_max_rate.as_ref().map(|x| str_to_dec(x)),
                    bid_min_rate: bid_min_rate.as_ref().map(|x| str_to_dec(x)),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_common_config(
        &mut self,
        sender: ProjectAccount,
        main_currency: &Option<Currency<TokenUnverified>>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        unbonding_period: &Option<u64>,
        collateral_min_value: &Option<u128>,
        min_available_liquidity_multiplier: &Option<&str>,
        price_update_period: &Option<u64>,
        is_marketplace_interface_enabled: &Option<bool>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::UpdateCommonConfig {
                    main_currency: main_currency.as_ref().map(|x| x.to_owned()),
                    bgl_currency: bgl_currency.as_ref().map(|x| x.to_owned()),
                    unbonding_period: unbonding_period.as_ref().map(|x| x.to_owned()),
                    collateral_min_value: collateral_min_value.map(Uint128::new),
                    min_available_liquidity_multiplier: min_available_liquidity_multiplier
                        .as_ref()
                        .map(|x| str_to_dec(x)),
                    price_update_period: price_update_period.as_ref().map(|x| x.to_owned()),
                    is_marketplace_interface_enabled: is_marketplace_interface_enabled.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_deposit_reserve_liquidity(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::DepositReserveLiquidity {};

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_lock(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::Lock {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_unlock(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::Unlock {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_distribute_funds(
        &mut self,
        sender: ProjectAccount,
        address_and_weight_list: &[(impl ToString, &str)],
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::DistributeFunds {
                    address_and_weight_list: address_and_weight_list
                        .iter()
                        .map(|(address, weight)| (address.to_string(), str_to_dec(weight)))
                        .collect(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_remove_collection(
        &mut self,
        sender: ProjectAccount,
        address: ProjectNft,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::RemoveCollection {
                    address: address.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_create_proposal(
        &mut self,
        sender: ProjectAccount,
        proposal: &Proposal<String, TokenUnverified>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::CreateProposal {
                    proposal: proposal.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_reject_proposal(
        &mut self,
        sender: ProjectAccount,
        id: u32,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::RejectProposal { id },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_accept_proposal(
        &mut self,
        sender: ProjectAccount,
        id: u32,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_lending_platform_address();
        let msg = &ExecuteMsg::AcceptProposal { id };

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_query_address_config(&self) -> StdResult<AddressConfig> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryAddressConfig {},
        )
    }

    #[track_caller]
    fn lending_platform_query_rate_config(&self) -> StdResult<RateConfig> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryRateConfig {},
        )
    }

    #[track_caller]
    fn lending_platform_query_common_config(&self) -> StdResult<CommonConfig> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCommonConfig {},
        )
    }

    #[track_caller]
    fn lending_platform_query_platform_revenue(&self) -> StdResult<Vec<Funds<Token>>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryPlatformRevenue {},
        )
    }

    #[track_caller]
    fn lending_platform_query_balances(&self) -> StdResult<Balances> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryBalances {},
        )
    }

    #[track_caller]
    fn lending_platform_query_unbonder_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryUnbondersResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryUnbonderList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_unbonder(&self, address: ProjectAccount) -> StdResult<Unbonder> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryUnbonder {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_borrower_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryBorrowersResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryBorrowerList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_borrower(&self, address: ProjectAccount) -> StdResult<Borrower> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryBorrower {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidator_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidatorsResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidatorList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidator(&self, address: ProjectAccount) -> StdResult<Liquidator> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidator {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collateral_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCollateralList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collateral(&self, address: ProjectNft) -> StdResult<Collateral> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCollateral {
                collection_address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_collection_address_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidationBidsByCollectionAddressList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_collection_address(
        &self,
        address: ProjectNft,
    ) -> StdResult<Vec<LiquidationBid>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidationBidsByCollectionAddress {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_liquidator_address_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddressList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_liquidator_address(
        &self,
        address: ProjectAccount,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddress {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_proposals(
        &self,
        start_after: &Option<u32>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryProposalsResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryProposals {
                start_after: start_after.to_owned(),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collection_list(
        &self,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollectionsResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCollectionList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collection(
        &self,
        address: ProjectNft,
    ) -> StdResult<Collection<Addr>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCollection {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_bgl_currency_to_main_currency_price(&self) -> StdResult<Decimal> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryBglCurrencyToMainCurrencyPrice {},
        )
    }

    #[track_caller]
    fn lending_platform_query_conditional_deposit_apr(
        &self,
        amount_to_deposit: &Option<u128>,
    ) -> StdResult<Decimal> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryConditionalDepositApr {
                amount_to_deposit: amount_to_deposit
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_ltv_list(
        &self,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<(Addr, Decimal)>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLtvList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_conditional_ltv(
        &self,
        borrower: ProjectAccount,
        amount_to_deposit: &Option<u128>,
        amount_to_borrow: &Option<u128>,
    ) -> StdResult<Decimal> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryConditionalLtv {
                borrower: borrower.to_string(),
                amount_to_deposit: amount_to_deposit
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
                amount_to_borrow: amount_to_borrow
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_total_available_to_borrow_liquidity(&self) -> StdResult<Uint128> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryTotalAvailableToBorrowLiquidity {},
        )
    }

    #[track_caller]
    fn lending_platform_query_available_to_borrow(
        &self,
        borrower: ProjectAccount,
        target_ltv: &Option<&str>,
    ) -> StdResult<Uint128> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryAvailableToBorrow {
                borrower: borrower.to_string(),
                target_ltv: target_ltv.as_ref().map(|x| str_to_dec(x)),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_amounts(&self) -> StdResult<QueryAmountsResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryAmounts {},
        )
    }

    #[track_caller]
    fn lending_platform_query_targets(
        &self,
        borrowers: &[impl ToString],
    ) -> StdResult<Vec<LiquidationItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryTargets {
                borrowers: borrowers.iter().map(|x| x.to_string()).collect(),
            },
        )
    }
}
