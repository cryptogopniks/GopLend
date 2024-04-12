use std::fmt::Debug;

use cosmwasm_std::{coin, to_json_binary, Addr, Coin, Empty, StdResult, Timestamp, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use serde::Serialize;
use strum::IntoEnumIterator;

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    error::parse_err,
    lending_platform::state::BGL_CURRENCY_DECIMALS,
    minter::{
        state::{CW20_NAME, CW20_SYMBOL},
        types::QueryCurrenciesFromCreatorResponse,
    },
};

use crate::helpers::{
    adapters::scheduler::kujira::AdapterSchedulerKujiraExtension,
    lending_platform::LendingPlatformExtension,
    minter::MinterExtension,
    oracle::OracleExtension,
    suite::{
        codes::WithCodes,
        types::{
            GetDecimals, ProjectAccount, ProjectAsset, ProjectCoin, ProjectNft, ProjectToken,
            WrappedResponse, DEFAULT_DECIMALS,
        },
    },
};

pub const INITIAL_LIQUIDITY: u128 = 100_000_000_000;

pub struct Project {
    pub app: App,
    pub logs: WrappedResponse,
    contract_counter: u16,

    // package code id
    cw20_base_code_id: u64,
    // cw721_base_code_id: u64,

    // contract code id
    lending_platform_code_id: u64,
    oracle_code_id: u64,
    minter_code_id: u64,
    adapter_scheduler_kujira_code_id: u64,

    // package address
    gopniks_address: Addr,
    pinjeons_address: Addr,

    // contract address
    lending_platform_address: Addr,
    oracle_address: Addr,
    minter_address: Addr,
    adapter_scheduler_kujira_address: Addr,
    //
    // other
}

impl Project {
    pub fn create_project_with_balances() -> Self {
        Self {
            app: Self::create_app_with_balances(),
            logs: WrappedResponse::Execute(Ok(AppResponse::default())),
            contract_counter: 0,

            cw20_base_code_id: 0,
            // cw721_base_code_id: 0,
            lending_platform_code_id: 0,
            oracle_code_id: 0,
            minter_code_id: 0,
            adapter_scheduler_kujira_code_id: 0,

            gopniks_address: Addr::unchecked(""),
            pinjeons_address: Addr::unchecked(""),

            lending_platform_address: Addr::unchecked(""),
            oracle_address: Addr::unchecked(""),
            minter_address: Addr::unchecked(""),
            adapter_scheduler_kujira_address: Addr::unchecked(""),
        }
    }

    pub fn new() -> Self {
        // create app and distribute coins to accounts
        let mut project = Self::create_project_with_balances();

        // register contracts code
        // packages
        let cw20_base_code_id = project.store_cw20_base_code();
        let cw721_base_code_id = project.store_cw721_base_code();

        // contracts
        let lending_platform_code_id = project.store_lending_platform_code();
        let oracle_code_id = project.store_oracle_code();
        let minter_code_id = project.store_minter_code();
        let adapter_scheduler_kujira_code_id = project.store_adapter_scheduler_kujira_code();

        // instantiate packages

        // DON'T CHANGE TOKEN INIT ORDER AS ITS ADDRESSES ARE HARDCODED IN ProjectToken ENUM
        for project_token in ProjectToken::iter() {
            project.instantiate_cw20_base_token(cw20_base_code_id, project_token);
        }

        let gopniks_address =
            project.instantiate_cw721_base_token(cw721_base_code_id, ProjectNft::Gopniks);
        let pinjeons_address =
            project.instantiate_cw721_base_token(cw721_base_code_id, ProjectNft::Pinjeons);

        // mint NFTs
        let token_id_list: Vec<u128> = vec![1, 2, 3];
        for collection in ProjectNft::iter() {
            for (i, recipient) in [
                ProjectAccount::Alice,
                ProjectAccount::Bob,
                ProjectAccount::John,
                ProjectAccount::Kate,
            ]
            .iter()
            .enumerate()
            {
                let nft_list: &Vec<u128> = &token_id_list
                    .iter()
                    .map(|x| x + (i as u128) * (token_id_list.len() as u128))
                    .collect();

                project.mint_nft(ProjectAccount::Owner, recipient, collection, nft_list);
            }
        }

        // instantiate goplend contracts

        let lending_platform_address = project.instantiate_lending_platform(
            lending_platform_code_id,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &Currency::new(
                &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                ProjectCoin::Stars.get_decimals(),
            ),
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        let oracle_address = project.instantiate_oracle(oracle_code_id, &None);
        let minter_address =
            project.instantiate_minter(minter_code_id, &None, &Some(cw20_base_code_id));
        let adapter_scheduler_kujira_address = project.instantiate_adapter_scheduler_kujira(
            adapter_scheduler_kujira_code_id,
            &None,
            &None,
            &None,
            &None,
        );

        project = Self {
            cw20_base_code_id,
            lending_platform_code_id,
            oracle_code_id,
            minter_code_id,
            adapter_scheduler_kujira_code_id,

            gopniks_address,
            pinjeons_address,

            lending_platform_address,
            oracle_address,
            minter_address,
            adapter_scheduler_kujira_address,

            ..project
        };

        // prepare contracts
        project
            .oracle_try_update_config(ProjectAccount::Admin, &Some(ProjectAccount::Scheduler))
            .unwrap();

        // create bgl_token
        project
            .minter_try_create_cw20(
                ProjectAccount::Admin,
                &project.get_lending_platform_address(),
                CW20_NAME,
                CW20_SYMBOL,
                &Some(BGL_CURRENCY_DECIMALS),
                &None,
            )
            .unwrap();

        let QueryCurrenciesFromCreatorResponse { currencies } = project
            .minter_query_currencies_by_creator(&project.get_lending_platform_address())
            .unwrap();
        let bgl_currency = &currencies[0];

        project
            .lending_platform_try_update_address_config(
                ProjectAccount::Admin,
                &Some(ProjectAccount::Owner),
                &Some(project.get_oracle_address()),
                &Some(project.get_minter_address()),
                &Some(project.get_adapter_scheduler_kujira_address()),
                &None,
            )
            .unwrap();

        project
            .lending_platform_try_update_common_config(
                ProjectAccount::Admin,
                &None,
                &Some(Currency::new(
                    &bgl_currency.clone().token.into(),
                    bgl_currency.decimals,
                )),
                &None,
                &None,
                &None,
                &None,
                &None,
            )
            .unwrap();

        project
            .adapter_scheduler_kujira_try_update_config(
                ProjectAccount::Admin,
                &Some(project.get_lending_platform_address()),
                &Some(vec![ProjectAccount::Scheduler.into()]),
                &None,
                &Some(0),
            )
            .unwrap();

        // add 100k STARS initial liquidity
        project
            .lending_platform_try_deposit_reserve_liquidity(
                ProjectAccount::Admin,
                INITIAL_LIQUIDITY,
                ProjectCoin::Stars,
            )
            .unwrap();

        project
    }

    // code id getters
    pub fn get_cw20_base_code_id(&self) -> u64 {
        self.cw20_base_code_id
    }

    pub fn get_lending_platform_code_id(&self) -> u64 {
        self.lending_platform_code_id
    }

    pub fn get_oracle_code_id(&self) -> u64 {
        self.oracle_code_id
    }

    pub fn get_minter_code_id(&self) -> u64 {
        self.minter_code_id
    }

    pub fn get_adapter_scheduler_kujira_code_id(&self) -> u64 {
        self.adapter_scheduler_kujira_code_id
    }

    // package address getters
    pub fn get_gopniks_address(&self) -> Addr {
        self.gopniks_address.clone()
    }

    pub fn get_pinjeons_address(&self) -> Addr {
        self.pinjeons_address.clone()
    }

    // contract address getters
    pub fn get_lending_platform_address(&self) -> Addr {
        self.lending_platform_address.clone()
    }

    pub fn get_oracle_address(&self) -> Addr {
        self.oracle_address.clone()
    }

    pub fn get_minter_address(&self) -> Addr {
        self.minter_address.clone()
    }

    pub fn get_adapter_scheduler_kujira_address(&self) -> Addr {
        self.adapter_scheduler_kujira_address.clone()
    }

    // utils
    pub fn increase_contract_counter(&mut self, step: u16) {
        self.contract_counter += step;
    }

    pub fn get_last_contract_address(&self) -> String {
        format!("contract{}", self.contract_counter)
    }

    pub fn get_timestamp(&self) -> Timestamp {
        self.app.block_info().time
    }

    pub fn reset_time(&mut self) {
        self.app.update_block(|block| {
            block.time = Timestamp::default().plus_seconds(1_000);
            block.height = 200;
        });
    }

    pub fn wait(&mut self, delay_s: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(delay_s);
            block.height += delay_s / 5;
        });
    }

    pub fn increase_allowances(
        &mut self,
        owner: ProjectAccount,
        spender: impl ToString,
        assets: &[(impl Into<Uint128> + Clone, ProjectToken)],
    ) {
        for (asset_amount, token) in assets {
            self.app
                .execute_contract(
                    owner.into(),
                    token.to_owned().into(),
                    &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                        spender: spender.to_string(),
                        amount: asset_amount.to_owned().into(),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
        }
    }

    pub fn increase_allowances_nft(
        &mut self,
        owner: ProjectAccount,
        spender: impl ToString,
        collection: ProjectNft,
    ) {
        self.app
            .execute_contract(
                owner.into(),
                collection.into(),
                &cw721_base::ExecuteMsg::ApproveAll::<Empty, Empty> {
                    operator: spender.to_string(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
    }

    pub fn mint_nft(
        &mut self,
        owner: ProjectAccount,
        recipient: impl ToString,
        collection: ProjectNft,
        token_id_list: &Vec<impl ToString>,
    ) {
        for token_id in token_id_list {
            let msg = &cw721_base::msg::ExecuteMsg::Mint::<Empty, Empty> {
                token_id: token_id.to_string(),
                owner: recipient.to_string(),
                token_uri: Some(format!("https://www.{:#?}.com", collection)),
                extension: Empty::default(),
            };

            self.app
                .execute_contract(owner.into(), collection.into(), msg, &[])
                .unwrap();
        }
    }

    pub fn query_all_nft(&self, owner: ProjectAccount) -> Vec<(ProjectNft, cw721::TokensResponse)> {
        let mut collection_and_tokens_response_list: Vec<(ProjectNft, cw721::TokensResponse)> =
            vec![];

        for collection in ProjectNft::iter() {
            let res: cw721::TokensResponse = self
                .app
                .wrap()
                .query_wasm_smart(
                    collection.to_string(),
                    &cw721::Cw721QueryMsg::Tokens {
                        owner: owner.to_string(),
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            collection_and_tokens_response_list.push((collection, res));
        }

        collection_and_tokens_response_list
    }

    pub fn query_balance(
        &self,
        account: ProjectAccount,
        token: &(impl Into<Token> + Clone),
    ) -> StdResult<u128> {
        let token: Token = token.to_owned().into();

        match token {
            Token::Native { denom } => Ok(self
                .app
                .wrap()
                .query_balance(account.to_string(), denom)?
                .amount
                .u128()),
            Token::Cw20 { address } => {
                let cw20::BalanceResponse { balance } = self.app.wrap().query_wasm_smart(
                    address.to_string(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: account.to_string(),
                    },
                )?;

                Ok(balance.u128())
            }
        }
    }

    pub fn query_all_balances(&self, account: ProjectAccount) -> StdResult<Vec<Funds<Token>>> {
        let mut funds_list: Vec<Funds<Token>> = vec![];

        for Coin { denom, amount } in self.app.wrap().query_all_balances(account.to_string())? {
            if !amount.is_zero() {
                funds_list.push(Funds::new(
                    amount,
                    &Currency::new(&Token::new_native(&denom), DEFAULT_DECIMALS),
                ));
            }
        }

        for token_cw20 in ProjectToken::iter() {
            let cw20::BalanceResponse { balance } = self.app.wrap().query_wasm_smart(
                token_cw20.to_string(),
                &cw20::Cw20QueryMsg::Balance {
                    address: account.to_string(),
                },
            )?;

            if !balance.is_zero() {
                funds_list.push(Funds::new(
                    balance,
                    &Currency::new(
                        &Token::new_cw20(&token_cw20.into()),
                        token_cw20.get_decimals(),
                    ),
                ));
            }
        }

        Ok(funds_list)
    }

    pub fn instantiate_contract(
        &mut self,
        code_id: u64,
        label: &str,
        init_msg: &impl Serialize,
    ) -> Addr {
        self.increase_contract_counter(1);

        self.app
            .instantiate_contract(
                code_id,
                ProjectAccount::Admin.into(),
                init_msg,
                &[],
                label,
                Some(ProjectAccount::Admin.to_string()),
            )
            .unwrap()
    }

    fn create_app_with_balances() -> App {
        App::new(|router, _api, storage| {
            for project_account in ProjectAccount::iter() {
                let funds: Vec<Coin> = ProjectCoin::iter()
                    .map(|project_coin| {
                        let amount = project_account.get_initial_funds_amount()
                            * 10u128.pow(project_coin.get_decimals() as u32);

                        coin(amount, project_coin.to_string())
                    })
                    .collect();

                router
                    .bank
                    .init_balance(storage, &project_account.into(), funds)
                    .unwrap();
            }
        })
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

pub fn assert_error<S: std::fmt::Debug + ToString>(
    subject: &S,
    err: impl ToString + Sized + Debug,
) {
    let expected_error_name = &format!("{:#?}", err);
    let expected_error_text = &err.to_string();

    speculoos::assert_that(subject).matches(|x| {
        let error = format!("{:#?}", x);

        error.contains(expected_error_name) || error.contains(expected_error_text)
    });
}

pub fn add_funds_to_exec_msg<T: Serialize + std::fmt::Debug>(
    project: &mut Project,
    sender: ProjectAccount,
    contract_address: &Addr,
    msg: &T,
    amount: impl Into<Uint128>,
    asset: impl Into<ProjectAsset>,
) -> StdResult<AppResponse> {
    let asset: ProjectAsset = asset.into();

    match asset {
        ProjectAsset::Coin(denom) => project
            .app
            .execute_contract(
                sender.into(),
                contract_address.to_owned(),
                msg,
                &[coin(
                    Into::<Uint128>::into(amount).u128(),
                    denom.to_string(),
                )],
            )
            .map_err(parse_err),
        ProjectAsset::Token(address) => {
            let wasm_msg = cw20::Cw20ExecuteMsg::Send {
                contract: contract_address.to_string(),
                amount: Into::<Uint128>::into(amount),
                msg: to_json_binary(msg).unwrap(),
            };

            project
                .app
                .execute_contract(sender.into(), address.into(), &wasm_msg, &[])
                .map_err(parse_err)
        }
    }
}

pub fn add_token_to_exec_msg<T: Serialize + std::fmt::Debug>(
    project: &mut Project,
    sender: ProjectAccount,
    contract_address: &Addr,
    msg: &T,
    amount: impl Into<Uint128>,
    asset: &Token,
) -> StdResult<AppResponse> {
    match asset {
        Token::Native { denom } => project
            .app
            .execute_contract(
                sender.into(),
                contract_address.to_owned(),
                msg,
                &[coin(
                    Into::<Uint128>::into(amount).u128(),
                    denom.to_string(),
                )],
            )
            .map_err(parse_err),
        Token::Cw20 { address } => {
            let wasm_msg = cw20::Cw20ExecuteMsg::Send {
                contract: contract_address.to_string(),
                amount: Into::<Uint128>::into(amount),
                msg: to_json_binary(msg).unwrap(),
            };

            project
                .app
                .execute_contract(sender.into(), address.to_owned(), &wasm_msg, &[])
                .map_err(parse_err)
        }
    }
}
