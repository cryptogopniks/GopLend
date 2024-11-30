use std::fmt::Debug;

use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, BlockInfo, Coin, Empty, StdResult, Timestamp, Uint128,
};
use cw_multi_test::{
    App, AppBuilder, AppResponse, BankSudo, Executor, MockAddressGenerator, MockApiBech32, SudoMsg,
    WasmKeeper,
};

use serde::Serialize;
use strum::IntoEnumIterator;

use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    error::parse_err,
    lending_platform::{state::BGL_CURRENCY_DECIMALS, types::TokenItem},
};

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    minter::MinterExtension,
    oracle::OracleExtension,
    scheduler::SchedulerExtension,
    suite::{
        codes::WithCodes,
        types::{
            CustomApp, GetDecimals, ProjectAccount, ProjectAsset, ProjectCoin, ProjectNft,
            ProjectToken, DEFAULT_DECIMALS,
        },
    },
};

pub const BGLSTARS_NAME: &str = "bonded GopLend STARS";
pub const BGLSTARS_SYMBOL: &str = "bglSTARS";
pub const BGLUSDC_NAME: &str = "bonded GopLend USDC";
pub const BGLUSDC_SYMBOL: &str = "bglUSDC";

pub const INITIAL_LIQUIDITY: u128 = 100_000_000_000;

pub struct Project {
    pub app: CustomApp,
    contract_counter: u16,

    // package code id
    cw20_base_code_id: u64,
    // cw721_base_code_id: u64,

    // contract code id
    lending_platform_code_id: u64,
    oracle_code_id: u64,
    minter_code_id: u64,
    scheduler_code_id: u64,
    market_maker_code_id: u64,

    // package address
    gopniks_address: Addr,
    pinjeons_address: Addr,

    // contract address
    lending_platform_address: Addr,
    oracle_address: Addr,
    minter_address: Addr,
    scheduler_address: Addr,
    market_maker_address: Addr,
    //
    // other
}

impl Project {
    pub fn create_project_with_balances() -> Self {
        Self {
            app: Self::create_app_with_balances(),
            contract_counter: 0,

            cw20_base_code_id: 0,
            // cw721_base_code_id: 0,
            lending_platform_code_id: 0,
            oracle_code_id: 0,
            minter_code_id: 0,
            scheduler_code_id: 0,
            market_maker_code_id: 0,

            gopniks_address: Addr::unchecked(""),
            pinjeons_address: Addr::unchecked(""),

            lending_platform_address: Addr::unchecked(""),
            oracle_address: Addr::unchecked(""),
            minter_address: Addr::unchecked(""),
            scheduler_address: Addr::unchecked(""),
            market_maker_address: Addr::unchecked(""),
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
        let scheduler_code_id = project.store_scheduler_code();
        let market_maker_code_id = project.store_market_maker_code();

        // instantiate packages

        // DON'T CHANGE TOKEN INIT ORDER AS ITS ADDRESSES ARE HARDCODED IN ProjectToken ENUM
        for project_token in ProjectToken::iter() {
            project.instantiate_cw20_base_token(cw20_base_code_id, project_token);
        }

        for _project_nft in ProjectNft::iter() {
            project.instantiate_cw721_base_token(cw721_base_code_id);
        }

        // mint NFTs
        let token_id_list: Vec<u128> = vec![1, 2, 3];
        for collection in ProjectNft::iter() {
            for (i, recipient) in [
                ProjectAccount::Alice,
                ProjectAccount::Bob,
                ProjectAccount::John,
                ProjectAccount::Kate,
                ProjectAccount::Ruby,
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
                &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                ProjectCoin::Usdc.get_decimals(),
            ),
            &None,
            &None,
            &None,
            &None,
        );
        let oracle_address = project.instantiate_oracle(oracle_code_id, &None, &None, &None, &None);
        let minter_address = project.instantiate_minter(
            minter_code_id,
            &None,
            &Some(cw20_base_code_id),
            &None,
            &None,
            &None,
        );
        let scheduler_address =
            project.instantiate_scheduler(scheduler_code_id, &None, &None, &None, &None);
        let market_maker_address = project.instantiate_market_maker(
            market_maker_code_id,
            &Some(ProjectAccount::Owner),
            &Some(vec![ProjectAccount::Scheduler]),
            &Some(lending_platform_address.clone()),
            &Some(oracle_address.clone()),
            &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
        );

        project = Self {
            cw20_base_code_id,

            lending_platform_code_id,
            oracle_code_id,
            minter_code_id,
            scheduler_code_id,
            market_maker_code_id,

            gopniks_address: Addr::unchecked(ProjectNft::Gopniks.to_string()),
            pinjeons_address: Addr::unchecked(ProjectNft::Pinjeons.to_string()),

            lending_platform_address,
            oracle_address,
            minter_address,
            scheduler_address,
            market_maker_address,

            ..project
        };

        // prepare contracts
        project
            .oracle_try_update_config(
                ProjectAccount::Admin,
                &Some(ProjectAccount::Admin),
                &Some(ProjectAccount::Owner),
                &Some(vec![ProjectAccount::Scheduler]),
                &Some(0),
                &None,
            )
            .unwrap();

        // create bgl_token
        project
            .minter_try_create_cw20(
                ProjectAccount::Admin,
                &None,
                &Some(vec![project.get_lending_platform_address()]),
                &None,
                &None,
                BGLUSDC_NAME,
                BGLUSDC_SYMBOL,
                &Some(BGL_CURRENCY_DECIMALS),
                &None,
            )
            .unwrap();

        let currency_info_list_by_owner = project
            .minter_query_currency_info_list_by_owner(ProjectAccount::Admin, 2, &None)
            .unwrap();
        let bgl_usdc_currency = &currency_info_list_by_owner[0].currency;

        project
            .lending_platform_try_update_address_config(
                ProjectAccount::Admin,
                &Some(ProjectAccount::Admin),
                &Some(ProjectAccount::Owner),
                &Some(project.get_oracle_address()),
                &Some(project.get_minter_address()),
                &Some(project.get_scheduler_address()),
                &Some(project.get_market_maker_address()),
            )
            .unwrap();

        project
            .lending_platform_try_update_common_config(
                ProjectAccount::Admin,
                &None,
                &Some(Currency::new(
                    &bgl_usdc_currency.clone().token.into(),
                    bgl_usdc_currency.decimals,
                )),
                &None,
                &None,
                &None,
            )
            .unwrap();

        project
            .scheduler_try_update_config(
                ProjectAccount::Admin,
                &None,
                &None,
                &Some(project.get_lending_platform_address()),
                &Some(vec![ProjectAccount::Scheduler.into()]),
                &None,
            )
            .unwrap();

        // add 100k USDC initial liquidity
        project
            .lending_platform_try_deposit_reserve_liquidity(
                ProjectAccount::Admin,
                INITIAL_LIQUIDITY,
                ProjectCoin::Usdc,
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

    pub fn get_scheduler_code_id(&self) -> u64 {
        self.scheduler_code_id
    }

    pub fn get_market_maker_code_id(&self) -> u64 {
        self.market_maker_code_id
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

    pub fn get_scheduler_address(&self) -> Addr {
        self.scheduler_address.clone()
    }

    pub fn get_market_maker_address(&self) -> Addr {
        self.market_maker_address.clone()
    }

    // utils
    pub fn addr(&self, acc: impl ToString) -> Addr {
        self.app.api().addr_make(&acc.to_string())
    }

    pub fn increase_contract_counter(&mut self, step: u16) {
        self.contract_counter += step;
    }

    pub fn get_last_contract_address(&self) -> String {
        format!("contract{}", self.contract_counter)
    }

    pub fn get_block_time(&self) -> u64 {
        self.app.block_info().time.seconds()
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

    pub fn set_chain_id(&mut self, chain_id: &str) {
        self.app.update_block(|block| {
            block.chain_id = chain_id.to_string();
        });
    }

    pub fn mint_native(&mut self, recipient: impl ToString, amount: u128, asset: ProjectCoin) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: recipient.to_string(),
                amount: coins(amount, asset.to_string()),
            }))
            .unwrap();
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

    pub fn transfer_nft(
        &mut self,
        owner: ProjectAccount,
        recipient: impl ToString,
        collection: ProjectNft,
        token_id: impl ToString,
    ) {
        let msg = &cw721_base::msg::ExecuteMsg::TransferNft::<Empty, Empty> {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        };

        self.app
            .execute_contract(owner.into(), collection.into(), msg, &[])
            .unwrap();
    }

    pub fn query_all_nft(&self, owner: impl ToString) -> Vec<(ProjectNft, cw721::TokensResponse)> {
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
        account: impl ToString,
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

    pub fn query_all_balances(&self, account: impl ToString) -> StdResult<Vec<Funds<Token>>> {
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
                Addr::unchecked(ProjectAccount::Admin.to_string()),
                init_msg,
                &[],
                label,
                Some(ProjectAccount::Admin.to_string()),
            )
            .unwrap()
    }

    pub fn create_app_with_balances() -> CustomApp {
        let block_info = App::default().block_info();

        AppBuilder::new_custom()
            .with_api(MockApiBech32::new("wasm"))
            .with_wasm(WasmKeeper::new().with_address_generator(MockAddressGenerator))
            .with_block(BlockInfo {
                height: block_info.height,
                time: block_info.time,
                chain_id: "wasm-1".to_string(),
            })
            .build(|router, _api, storage| {
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

pub fn to_string_vec(str_vec: &[&str]) -> Vec<String> {
    str_vec.iter().map(|x| x.to_string()).collect()
}

pub fn to_token_item_list(str_vec: &[&str], date: u64) -> Vec<TokenItem> {
    str_vec
        .iter()
        .map(|x| TokenItem {
            id: x.to_string(),
            creation_date: date,
        })
        .collect()
}

pub fn to_token_item(id: &str) -> TokenItem {
    TokenItem {
        id: id.to_string(),
        creation_date: 0,
    }
}
