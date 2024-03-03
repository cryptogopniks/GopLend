use cosmwasm_std::{Decimal, StdResult, Uint128};
use cw_multi_test::{AppResponse, Executor};

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::str_to_dec,
    error::parse_err,
    lending_platform::{
        msg::{
            ExecuteMsg, QueryBorrowersResponseItem, QueryCollateralsResponseItem,
            QueryCollectionsResponseItem, QueryLiquidationBidsResponseItem,
            QueryLiquidatorsResponseItem, QueryMsg, QueryProposalsResponseItem,
            QueryUnbondersResponseItem, SudoMsg,
        },
        types::{Balances, CollectionInfo, Config, Proposal},
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

    fn lending_platform_try_process(&mut self) -> StdResult<AppResponse>;

    fn lending_platform_try_update_config(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        oracle: &Option<String>,
        main_currency: &Option<Currency<TokenUnverified>>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        cw20_base_code_id: &Option<u64>,
        unbonding_period: &Option<u64>,
        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,
        pagination_amount: &Option<u32>,
        collateral_min_value: &Option<u128>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_create_bgl_token(
        &mut self,
        sender: ProjectAccount,
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

    fn lending_platform_query_config(&self) -> StdResult<Config>;

    fn lending_platform_query_platform_revenue(&self) -> StdResult<Vec<Funds<Token>>>;

    fn lending_platform_query_balances(&self) -> StdResult<Balances>;

    fn lending_platform_query_unbonders(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryUnbondersResponseItem>>;

    fn lending_platform_query_borrowers(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryBorrowersResponseItem>>;

    fn lending_platform_query_liquidators(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryLiquidatorsResponseItem>>;

    fn lending_platform_query_collaterals(
        &self,
        collection_addresses: &Option<Vec<ProjectNft>>,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_collection_addresses(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryLiquidationBidsResponseItem>>;

    fn lending_platform_query_liquidation_bids_by_liquidator_addresses(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryLiquidationBidsResponseItem>>;

    fn lending_platform_query_proposals(
        &self,
        last_amount: Option<u32>,
    ) -> StdResult<Vec<QueryProposalsResponseItem>>;

    fn lending_platform_query_collections(
        &self,
        addresses: &Option<Vec<ProjectNft>>,
    ) -> StdResult<Vec<QueryCollectionsResponseItem>>;

    fn lending_platform_query_bgl_currency_to_main_currency_price(&self) -> StdResult<Decimal>;

    fn lending_platform_query_deposit_apr(&self) -> StdResult<Decimal>;

    fn lending_platform_query_ltv(&self, borrower: ProjectAccount) -> StdResult<Decimal>;
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
    fn lending_platform_try_process(&mut self) -> StdResult<AppResponse> {
        self.app
            .wasm_sudo(self.get_lending_platform_address(), &SudoMsg::BeginBlock {})
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_update_config(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        oracle: &Option<String>,
        main_currency: &Option<Currency<TokenUnverified>>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        cw20_base_code_id: &Option<u64>,
        unbonding_period: &Option<u64>,
        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,
        pagination_amount: &Option<u32>,
        collateral_min_value: &Option<u128>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::UpdateConfig {
                    owner: owner.as_ref().map(|x| x.to_string()),
                    oracle: oracle.as_ref().map(|x| x.to_string()),
                    main_currency: main_currency.as_ref().map(|x| x.to_owned()),
                    bgl_currency: bgl_currency.as_ref().map(|x| x.to_owned()),
                    cw20_base_code_id: cw20_base_code_id.as_ref().map(|x| x.to_owned()),
                    unbonding_period: unbonding_period.as_ref().map(|x| x.to_owned()),
                    borrow_apr: borrow_apr.as_ref().map(|x| str_to_dec(x)),
                    borrow_fee_rate: borrow_fee_rate.as_ref().map(|x| str_to_dec(x)),
                    liquidation_fee_rate: liquidation_fee_rate.as_ref().map(|x| str_to_dec(x)),
                    discount_min_rate: discount_min_rate.as_ref().map(|x| str_to_dec(x)),
                    discount_max_rate: discount_max_rate.as_ref().map(|x| str_to_dec(x)),
                    bid_min_rate: bid_min_rate.as_ref().map(|x| str_to_dec(x)),
                    pagination_amount: pagination_amount.as_ref().map(|x| x.to_owned()),
                    collateral_min_value: collateral_min_value.map(|x| Uint128::new(x)),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_create_bgl_token(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_lending_platform_address(),
                &ExecuteMsg::CreateBglToken {},
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
    fn lending_platform_query_config(&self) -> StdResult<Config> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryConfig {},
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
    fn lending_platform_query_unbonders(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryUnbondersResponseItem>> {
        let addresses = addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryUnbonders { addresses },
        )
    }

    #[track_caller]
    fn lending_platform_query_borrowers(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryBorrowersResponseItem>> {
        let addresses = addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryBorrowers { addresses },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidators(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryLiquidatorsResponseItem>> {
        let addresses = addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidators { addresses },
        )
    }

    #[track_caller]
    fn lending_platform_query_collaterals(
        &self,
        collection_addresses: &Option<Vec<ProjectNft>>,
    ) -> StdResult<Vec<QueryCollateralsResponseItem>> {
        let collection_addresses = collection_addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCollaterals {
                collection_addresses,
            },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_collection_addresses(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryLiquidationBidsResponseItem>> {
        let addresses = addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidationBidsByCollectionAddresses { addresses },
        )
    }

    #[track_caller]
    fn lending_platform_query_liquidation_bids_by_liquidator_addresses(
        &self,
        addresses: &Option<Vec<ProjectAccount>>,
    ) -> StdResult<Vec<QueryLiquidationBidsResponseItem>> {
        let addresses = addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLiquidationBidsByLiquidatorAddresses { addresses },
        )
    }

    #[track_caller]
    fn lending_platform_query_proposals(
        &self,
        last_amount: Option<u32>,
    ) -> StdResult<Vec<QueryProposalsResponseItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryProposals { last_amount },
        )
    }

    #[track_caller]
    fn lending_platform_query_collections(
        &self,
        addresses: &Option<Vec<ProjectNft>>,
    ) -> StdResult<Vec<QueryCollectionsResponseItem>> {
        let addresses = addresses
            .as_ref()
            .map(|x| x.iter().map(|y| y.to_string()).collect());

        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryCollections { addresses },
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
    fn lending_platform_query_deposit_apr(&self) -> StdResult<Decimal> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryDepositApr {},
        )
    }

    #[track_caller]
    fn lending_platform_query_ltv(&self, borrower: ProjectAccount) -> StdResult<Decimal> {
        self.app.wrap().query_wasm_smart(
            self.get_lending_platform_address(),
            &QueryMsg::QueryLtv {
                borrower: borrower.to_string(),
            },
        )
    }
}
