use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    adapters::{dex::stargaze::types::Route, marketplace::stargaze::types::PlatformType},
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
            QueryUserInfoResponse,
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
        platform_type: PlatformType,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_unbond(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
        asset: &Token,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_withdraw(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_deposit_collateral(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_withdraw_collateral(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_borrow(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_repay(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_place_bid(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
        discount: &str,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_remove_bid(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collection_addresses: &[String],
        creation_date: u64,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn lending_platform_try_update_bid(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
        creation_date: u64,
        amount: u128,
        discount: &str,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_liquidate(
        &mut self,
        sender: &Addr,
        platform_type: PlatformType,
        targets: &[LiquidationItem],
        route: &Route,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn lending_platform_try_update_address_config(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        oracle: &Option<Addr>,
        minter: &Option<Addr>,
        adapter_scheduler: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn lending_platform_try_update_rate_config(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
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
        platform_type: PlatformType,
        main_currency: &Option<Currency<TokenUnverified>>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        unbonding_period: &Option<u64>,
        collateral_min_value: &Option<u128>,
        borrowers_reserve_fraction_ratio: &Option<&str>,
        is_marketplace_interface_enabled: &Option<bool>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_deposit_reserve_liquidity(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_withdraw_reserve_liquidity(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_lock(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_unlock(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_distribute_funds(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        address_and_weight_list: &[(impl ToString, &str)],
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_remove_collection(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_create_proposal(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        proposal: &Proposal<String, TokenUnverified>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_reject_proposal(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        id: u32,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_accept_proposal(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        id: u32,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_query_address_config(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<AddressConfig>;

    fn lending_platform_query_rate_config(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<RateConfig>;

    fn lending_platform_query_common_config(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<CommonConfig>;

    fn lending_platform_query_platform_revenue(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<Vec<Funds<Token>>>;

    fn lending_platform_query_balances(&self, platform_type: PlatformType) -> StdResult<Balances>;

    fn lending_platform_query_unbonder_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryUnbondersResponseItem>>;

    fn lending_platform_query_unbonder(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Unbonder>;

    fn lending_platform_query_borrower_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryBorrowersResponseItem>>;

    fn lending_platform_query_borrower(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Borrower>;

    fn lending_platform_query_liquidator_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidatorsResponseItem>>;

    fn lending_platform_query_liquidator(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Liquidator>;

    fn lending_platform_query_collateral_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>>;

    fn lending_platform_query_collateral(
        &self,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<Vec<Collateral>>;

    fn lending_platform_query_collateral_by_owner(
        &self,
        platform_type: PlatformType,
        owner: ProjectAccount,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_collection_address_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_collection_address(
        &self,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<Vec<LiquidationBid>>;

    fn lending_platform_query_liquidation_bids_by_liquidator_address_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_liquidator_address(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>>;

    fn lending_platform_query_proposals(
        &self,
        platform_type: PlatformType,
        start_after: &Option<u32>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryProposalsResponseItem>>;

    fn lending_platform_query_collection_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollectionsResponseItem>>;

    fn lending_platform_query_collection(
        &self,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<Collection<Addr>>;

    fn lending_platform_query_bgl_currency_to_main_currency_price(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<Decimal>;

    fn lending_platform_query_conditional_deposit_apr(
        &self,
        platform_type: PlatformType,
        amount_to_deposit: &Option<u128>,
        amount_to_withdraw: &Option<u128>,
    ) -> StdResult<Decimal>;

    fn lending_platform_query_ltv_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<(Addr, Option<Decimal>)>>;

    fn lending_platform_query_conditional_ltv(
        &self,
        platform_type: PlatformType,
        borrower: ProjectAccount,
        amount_to_deposit: &Option<u128>,
        amount_to_withdraw: &Option<u128>,
        amount_to_borrow: &Option<u128>,
        amount_to_repay: &Option<u128>,
    ) -> StdResult<Decimal>;

    fn lending_platform_query_total_available_to_borrow_liquidity(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<Uint128>;

    fn lending_platform_query_available_to_borrow(
        &self,
        platform_type: PlatformType,
        borrower: ProjectAccount,
        target_ltv: &Option<&str>,
    ) -> StdResult<Uint128>;

    fn lending_platform_query_amounts(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<QueryAmountsResponse>;

    fn lending_platform_query_user_info(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<QueryUserInfoResponse>;
}

impl LendingPlatformExtension for Project {
    #[track_caller]
    fn lending_platform_try_deposit(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };
        let msg = &ExecuteMsg::Deposit {};

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_unbond(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
        asset: &Token,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };
        let msg = &ExecuteMsg::Unbond {};

        add_token_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_withdraw(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
                &ExecuteMsg::Withdraw {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_deposit_collateral(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        amount: u128,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        let msg = &ExecuteMsg::Repay {};

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_place_bid(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
        discount: &str,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        let msg = &ExecuteMsg::PlaceBid {
            collections: collections.to_owned(),
            discount: str_to_dec(discount),
        };

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_update_bid(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collections: &[CollectionInfo<String>],
        creation_date: u64,
        amount: u128,
        discount: &str,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        let msg = &ExecuteMsg::UpdateBid {
            collections: collections.to_owned(),
            creation_date,
            amount: Uint128::from(amount),
            discount: str_to_dec(discount),
        };

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_remove_bid(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        collection_addresses: &[String],
        creation_date: u64,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
                &ExecuteMsg::RemoveBid {
                    collection_addresses: collection_addresses.to_owned(),
                    creation_date,
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_liquidate(
        &mut self,
        sender: &Addr,
        platform_type: PlatformType,
        targets: &[LiquidationItem],
        route: &Route,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.to_owned(),
                contract_address,
                &ExecuteMsg::Liquidate {
                    targets: targets.to_owned(),
                    route: route.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_accept_admin_role(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_address_config(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        admin: &Option<ProjectAccount>,
        worker: &Option<ProjectAccount>,
        oracle: &Option<Addr>,
        minter: &Option<Addr>,
        adapter_scheduler: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
                &ExecuteMsg::UpdateAddressConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                    minter: minter.as_ref().map(|x| x.to_string()),
                    adapter_scheduler: adapter_scheduler.as_ref().map(|x| x.to_string()),
                    adapter_marketplace: adapter_marketplace.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_rate_config(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        main_currency: &Option<Currency<TokenUnverified>>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        unbonding_period: &Option<u64>,
        collateral_min_value: &Option<u128>,
        borrowers_reserve_fraction_ratio: &Option<&str>,
        is_marketplace_interface_enabled: &Option<bool>,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
                &ExecuteMsg::UpdateCommonConfig {
                    main_currency: main_currency.as_ref().map(|x| x.to_owned()),
                    bgl_currency: bgl_currency.as_ref().map(|x| x.to_owned()),
                    unbonding_period: unbonding_period.as_ref().map(|x| x.to_owned()),
                    collateral_min_value: collateral_min_value.map(Uint128::new),
                    borrowers_reserve_fraction_ratio: borrowers_reserve_fraction_ratio
                        .as_ref()
                        .map(|x| str_to_dec(x)),
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
        platform_type: PlatformType,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        let msg = &ExecuteMsg::DepositReserveLiquidity {};

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_try_withdraw_reserve_liquidity(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        amount: u128,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address.to_owned(),
                &ExecuteMsg::WithdrawReserveLiquidity {
                    amount: Uint128::new(amount),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_lock(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(sender.into(), contract_address, &ExecuteMsg::Lock {}, &[])
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_unlock(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(sender.into(), contract_address, &ExecuteMsg::Unlock {}, &[])
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_distribute_funds(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        address_and_weight_list: &[(impl ToString, &str)],
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        proposal: &Proposal<String, TokenUnverified>,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
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
        platform_type: PlatformType,
        id: u32,
    ) -> StdResult<AppResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .execute_contract(
                sender.into(),
                contract_address,
                &ExecuteMsg::RejectProposal { id },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_accept_proposal(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        id: u32,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };
        let msg = &ExecuteMsg::AcceptProposal { id };

        add_funds_to_exec_msg(self, sender, contract_address, msg, amount, asset)
    }

    #[track_caller]
    fn lending_platform_query_address_config(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<AddressConfig> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .wrap()
            .query_wasm_smart(contract_address, &QueryMsg::QueryAddressConfig {})
    }

    #[track_caller]
    fn lending_platform_query_rate_config(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<RateConfig> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .wrap()
            .query_wasm_smart(contract_address, &QueryMsg::QueryRateConfig {})
    }

    #[track_caller]
    fn lending_platform_query_common_config(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<CommonConfig> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .wrap()
            .query_wasm_smart(contract_address, &QueryMsg::QueryCommonConfig {})
    }

    #[track_caller]
    fn lending_platform_query_platform_revenue(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<Vec<Funds<Token>>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .wrap()
            .query_wasm_smart(contract_address, &QueryMsg::QueryPlatformRevenue {})
    }

    #[track_caller]
    fn lending_platform_query_balances(&self, platform_type: PlatformType) -> StdResult<Balances> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .wrap()
            .query_wasm_smart(contract_address, &QueryMsg::QueryBalances {})
    }

    #[track_caller]
    fn lending_platform_query_unbonder_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryUnbondersResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryUnbonderList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_unbonder(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Unbonder> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryUnbonder {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_borrower_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryBorrowersResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryBorrowerList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_borrower(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Borrower> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryBorrower {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidator_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidatorsResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLiquidatorList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidator(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Liquidator> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLiquidator {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collateral_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryCollateralList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collateral(
        &self,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<Vec<Collateral>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryCollateral {
                collection_address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collateral_by_owner(
        &self,
        platform_type: PlatformType,
        owner: ProjectAccount,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryCollateralByOwner {
                owner: owner.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_collection_address_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLiquidationBidsByCollectionAddressList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_collection_address(
        &self,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<Vec<LiquidationBid>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLiquidationBidsByCollectionAddress {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_liquidator_address_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryLiquidationBidsByLiquidatorAddressListResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddressList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_liquidator_address(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<Vec<QueryLiquidationBidsByCollectionAddressListResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddress {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_proposals(
        &self,
        platform_type: PlatformType,
        start_after: &Option<u32>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryProposalsResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryProposals {
                start_after: start_after.to_owned(),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collection_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectNft>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<QueryCollectionsResponseItem>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryCollectionList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_collection(
        &self,
        platform_type: PlatformType,
        address: ProjectNft,
    ) -> StdResult<Collection<Addr>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryCollection {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_bgl_currency_to_main_currency_price(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<Decimal> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryBglCurrencyToMainCurrencyPrice {},
        )
    }

    #[track_caller]
    fn lending_platform_query_conditional_deposit_apr(
        &self,
        platform_type: PlatformType,
        amount_to_deposit: &Option<u128>,
        amount_to_withdraw: &Option<u128>,
    ) -> StdResult<Decimal> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryConditionalDepositApr {
                amount_to_deposit: amount_to_deposit
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
                amount_to_withdraw: amount_to_withdraw
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_ltv_list(
        &self,
        platform_type: PlatformType,
        start_after: &Option<ProjectAccount>,
        limit: &Option<u32>,
    ) -> StdResult<Vec<(Addr, Option<Decimal>)>> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryLtvList {
                start_after: start_after.as_ref().map(|x| x.to_string()),
                limit: limit.to_owned(),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_conditional_ltv(
        &self,
        platform_type: PlatformType,
        borrower: ProjectAccount,
        amount_to_deposit: &Option<u128>,
        amount_to_withdraw: &Option<u128>,
        amount_to_borrow: &Option<u128>,
        amount_to_repay: &Option<u128>,
    ) -> StdResult<Decimal> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryConditionalLtv {
                borrower: borrower.to_string(),
                amount_to_deposit: amount_to_deposit
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
                amount_to_withdraw: amount_to_withdraw
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
                amount_to_borrow: amount_to_borrow
                    .as_ref()
                    .map(|x| Uint128::new(x.to_owned())),
                amount_to_repay: amount_to_repay.as_ref().map(|x| Uint128::new(x.to_owned())),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_total_available_to_borrow_liquidity(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<Uint128> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryTotalAvailableToBorrowLiquidity {},
        )
    }

    #[track_caller]
    fn lending_platform_query_available_to_borrow(
        &self,
        platform_type: PlatformType,
        borrower: ProjectAccount,
        target_ltv: &Option<&str>,
    ) -> StdResult<Uint128> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryAvailableToBorrow {
                borrower: borrower.to_string(),
                target_ltv: target_ltv.as_ref().map(|x| str_to_dec(x)),
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_amounts(
        &self,
        platform_type: PlatformType,
    ) -> StdResult<QueryAmountsResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app
            .wrap()
            .query_wasm_smart(contract_address, &QueryMsg::QueryAmounts {})
    }

    #[track_caller]
    fn lending_platform_query_user_info(
        &self,
        platform_type: PlatformType,
        address: ProjectAccount,
    ) -> StdResult<QueryUserInfoResponse> {
        let contract_address = match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        self.app.wrap().query_wasm_smart(
            contract_address,
            &QueryMsg::QueryUserInfo {
                address: address.to_string(),
            },
        )
    }
}
