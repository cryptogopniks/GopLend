use cosmwasm_std::{coin, Addr, StdResult, Uint128};
use cw_multi_test::{AppResponse, Executor};

use cw20_base::msg::InstantiateMarketingInfo;

use goplend_base::{
    adapters::marketplace::stargaze::types::PlatformType,
    assets::{Currency, Token},
    error::parse_err,
    minter::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, Metadata, QueryCurrenciesFromCreatorResponse},
    },
};

use crate::helpers::suite::{
    core::Project,
    types::{ProjectAccount, ProjectCoin},
};

use super::suite::core::add_token_to_exec_msg;

pub trait MinterExtension {
    fn minter_try_create_native(
        &mut self,
        sender: ProjectAccount,
        token_owner: &Addr,
        subdenom: ProjectCoin,
        decimals: &Option<u8>,
        payment: (u128, ProjectCoin),
    ) -> StdResult<AppResponse>;

    fn minter_try_create_cw20(
        &mut self,
        sender: ProjectAccount,
        token_owner: &Addr,
        name: &str,
        symbol: &str,
        decimals: &Option<u8>,
        marketing: &Option<InstantiateMarketingInfo>,
    ) -> StdResult<AppResponse>;

    fn minter_try_mint(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        amount: u128,
        recipient: &Addr,
    ) -> StdResult<AppResponse>;

    fn minter_try_burn(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        token: &Token,
        amount: u128,
    ) -> StdResult<AppResponse>;

    fn minter_try_set_metadata_native(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        metadata: &Metadata,
    ) -> StdResult<AppResponse>;

    fn minter_try_update_marketing_cw20(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        project: &Option<&str>,
        description: &Option<&str>,
        marketing: &Option<&str>,
    ) -> StdResult<AppResponse>;

    fn minter_try_upload_logo_cw20(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        logo: &cw20::Logo,
    ) -> StdResult<AppResponse>;

    fn minter_try_change_admin_native(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
    ) -> StdResult<AppResponse>;

    fn minter_try_update_minter_cw20(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        new_minter: &Addr,
    ) -> StdResult<AppResponse>;

    fn minter_try_register_currency(
        &mut self,
        sender: ProjectAccount,
        currency: &Currency<Token>,
        creator: &Addr,
    ) -> StdResult<AppResponse>;

    fn minter_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn minter_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<Addr>,
        cw20_base_code_id: &Option<u64>,
    ) -> StdResult<AppResponse>;

    fn minter_query_currencies_by_creator(
        &self,
        creator: &Addr,
    ) -> StdResult<QueryCurrenciesFromCreatorResponse>;

    fn minter_query_config(&self) -> StdResult<Config>;
}

impl MinterExtension for Project {
    #[track_caller]
    fn minter_try_create_native(
        &mut self,
        sender: ProjectAccount,
        token_owner: &Addr,
        subdenom: ProjectCoin,
        decimals: &Option<u8>,
        payment: (u128, ProjectCoin),
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::CreateNative {
                    token_owner: token_owner.to_string(),
                    subdenom: subdenom.to_string(),
                    decimals: decimals.to_owned(),
                },
                &[coin(payment.0, payment.1.to_string())],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_create_cw20(
        &mut self,
        sender: ProjectAccount,
        token_owner: &Addr,
        name: &str,
        symbol: &str,
        decimals: &Option<u8>,
        marketing: &Option<InstantiateMarketingInfo>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::CreateCw20 {
                    token_owner: token_owner.to_string(),
                    name: name.to_string(),
                    symbol: symbol.to_string(),
                    decimals: decimals.to_owned(),
                    marketing: marketing.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_mint(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        amount: u128,
        recipient: &Addr,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::Mint {
                    token: token.to_owned().into(),
                    amount: Uint128::new(amount),
                    recipient: recipient.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_burn(
        &mut self,
        sender: ProjectAccount,
        platform_type: PlatformType,
        token: &Token,
        amount: u128,
    ) -> StdResult<AppResponse> {
        let contract_address = &match platform_type {
            PlatformType::Main => self.get_lending_platform_main_address(),
            PlatformType::Stable => self.get_lending_platform_stable_address(),
        };

        let msg = &ExecuteMsg::Burn {};

        add_token_to_exec_msg(self, sender, contract_address, msg, amount, token)
    }

    #[track_caller]
    fn minter_try_set_metadata_native(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        metadata: &Metadata,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::SetMetadataNative {
                    token: token.to_owned().into(),
                    metadata: metadata.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_marketing_cw20(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        project: &Option<&str>,
        description: &Option<&str>,
        marketing: &Option<&str>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateMarketingCw20 {
                    token: token.to_owned().into(),
                    project: project.as_ref().map(|x| x.to_string()),
                    description: description.as_ref().map(|x| x.to_string()),
                    marketing: marketing.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_upload_logo_cw20(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        logo: &cw20::Logo,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UploadLogoCw20 {
                    token: token.to_owned().into(),
                    logo: logo.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_change_admin_native(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::ChangeAdminNative {
                    token: token.to_owned().into(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_minter_cw20(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        new_minter: &Addr,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateMinterCw20 {
                    token: token.to_owned().into(),
                    new_minter: new_minter.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_register_currency(
        &mut self,
        sender: ProjectAccount,
        currency: &Currency<Token>,
        creator: &Addr,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::RegisterCurrency {
                    currency: Currency::new(&currency.clone().token.into(), currency.decimals),
                    creator: creator.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        worker: &Option<Addr>,
        cw20_base_code_id: &Option<u64>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    worker: worker.as_ref().map(|x| x.to_string()),
                    cw20_base_code_id: cw20_base_code_id.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_query_currencies_by_creator(
        &self,
        creator: &Addr,
    ) -> StdResult<QueryCurrenciesFromCreatorResponse> {
        self.app.wrap().query_wasm_smart(
            self.get_minter_address(),
            &QueryMsg::QueryCurrenciesByCreator {
                creator: creator.to_string(),
            },
        )
    }

    #[track_caller]
    fn minter_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_minter_address(), &QueryMsg::QueryConfig {})
    }
}
