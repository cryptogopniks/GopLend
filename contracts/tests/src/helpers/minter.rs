use cosmwasm_std::{coins, Addr, StdResult, Uint128};
use cw20::Logo;
use cw_gopniks::assets::Token;
use cw_multi_test::{AppResponse, Executor};

use cw20_base::msg::InstantiateMarketingInfo;

use goplend_base::{
    error::parse_err,
    minter::{
        msg::{ExecuteMsg, QueryMsg},
        types::{Config, CurrencyInfo, Metadata},
    },
};

use crate::helpers::suite::{
    core::{add_token_to_exec_msg, Project},
    types::{ProjectAccount, ProjectCoin},
};

pub trait MinterExtension {
    fn minter_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn minter_try_accept_token_owner_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse>;

    fn minter_try_pause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn minter_try_unpause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn minter_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        whitelist: &Option<Vec<ProjectAccount>>,
        cw20_code_id: &Option<u64>,
        permissionless_token_creation: &Option<bool>,
        permissionless_token_registration: &Option<bool>,
        max_tokens_per_owner: &Option<u16>,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn minter_try_create_native(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        subdenom: ProjectCoin,
        decimals: &Option<u8>,
        payment: (u128, ProjectCoin),
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn minter_try_create_cw20(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        cw20_code_id: &Option<u64>,
        name: &str,
        symbol: &str,
        decimals: &Option<u8>,
        marketing: &Option<InstantiateMarketingInfo>,
    ) -> StdResult<AppResponse>;

    fn minter_try_register_native(
        &mut self,
        sender: ProjectAccount,
        denom: &str,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        decimals: &Option<u8>,
    ) -> StdResult<AppResponse>;

    #[allow(clippy::too_many_arguments)]
    fn minter_try_register_cw20(
        &mut self,
        sender: ProjectAccount,
        address: &Addr,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        cw20_code_id: &Option<u64>,
        decimals: &Option<u8>,
    ) -> StdResult<AppResponse>;

    fn minter_try_update_currency_info(
        &mut self,
        sender: ProjectAccount,
        denom_or_address: &str,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
    ) -> StdResult<AppResponse>;

    fn minter_try_update_metadata_native(
        &mut self,
        sender: ProjectAccount,
        denom: &str,
        metadata: &Metadata,
    ) -> StdResult<AppResponse>;

    fn minter_try_update_metadata_cw20(
        &mut self,
        sender: ProjectAccount,
        address: &Addr,
        project: &Option<&str>,
        description: &Option<&str>,
        logo: &Option<Logo>,
    ) -> StdResult<AppResponse>;

    fn minter_try_exclude_native(
        &mut self,
        sender: ProjectAccount,
        denom: &str,
    ) -> StdResult<AppResponse>;

    fn minter_try_exclude_cw20(
        &mut self,
        sender: ProjectAccount,
        address: &Addr,
    ) -> StdResult<AppResponse>;

    fn minter_try_mint(
        &mut self,
        sender: ProjectAccount,
        denom_or_address: &str,
        amount: u128,
        recipient: &Option<&Addr>,
    ) -> StdResult<AppResponse>;

    fn minter_try_burn(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        amount: u128,
    ) -> StdResult<AppResponse>;

    fn minter_query_config(&self) -> StdResult<Config>;

    fn minter_query_currency_info(&self, denom_or_address: &str) -> StdResult<CurrencyInfo>;

    fn minter_query_currency_info_list(
        &self,
        amount: u32,
        start_after: &Option<&str>,
    ) -> StdResult<Vec<CurrencyInfo>>;

    fn minter_query_currency_info_list_by_owner(
        &self,
        owner: ProjectAccount,
        amount: u32,
        start_after: &Option<&str>,
    ) -> StdResult<Vec<CurrencyInfo>>;

    fn minter_query_token_count_list(
        &self,
        amount: u32,
        start_after: &Option<&Addr>,
    ) -> StdResult<Vec<(Addr, u16)>>;
}

impl MinterExtension for Project {
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
    fn minter_try_accept_token_owner_role(
        &mut self,
        sender: ProjectAccount,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::AcceptTokenOwnerRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_pause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::Pause {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_unpause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::Unpause {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: &Option<ProjectAccount>,
        whitelist: &Option<Vec<ProjectAccount>>,
        cw20_code_id: &Option<u64>,
        permissionless_token_creation: &Option<bool>,
        permissionless_token_registration: &Option<bool>,
        max_tokens_per_owner: &Option<u16>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.as_ref().map(|x| x.to_string()),
                    whitelist: whitelist
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect::<Vec<String>>()),
                    cw20_code_id: cw20_code_id.to_owned(),
                    permissionless_token_creation: permissionless_token_creation.to_owned(),
                    permissionless_token_registration: permissionless_token_registration.to_owned(),
                    max_tokens_per_owner: max_tokens_per_owner.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_create_native(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        subdenom: ProjectCoin,
        decimals: &Option<u8>,
        payment: (u128, ProjectCoin),
    ) -> StdResult<AppResponse> {
        let (amount, denom) = payment;

        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::CreateNative {
                    owner: owner.as_ref().map(|x| x.to_string()),
                    whitelist: whitelist
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect::<Vec<String>>()),
                    permissionless_burning: permissionless_burning.to_owned(),
                    subdenom: subdenom.to_string(),
                    decimals: decimals.to_owned(),
                },
                &coins(amount, denom.to_string()),
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_create_cw20(
        &mut self,
        sender: ProjectAccount,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        cw20_code_id: &Option<u64>,
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
                    owner: owner.as_ref().map(|x| x.to_string()),
                    whitelist: whitelist
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect::<Vec<String>>()),
                    permissionless_burning: permissionless_burning.to_owned(),
                    cw20_code_id: cw20_code_id.to_owned(),
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
    fn minter_try_register_native(
        &mut self,
        sender: ProjectAccount,
        denom: &str,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        decimals: &Option<u8>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::RegisterNative {
                    denom: denom.to_string(),
                    owner: owner.as_ref().map(|x| x.to_string()),
                    whitelist: whitelist
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect::<Vec<String>>()),
                    permissionless_burning: permissionless_burning.to_owned(),
                    decimals: decimals.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_register_cw20(
        &mut self,
        sender: ProjectAccount,
        address: &Addr,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
        cw20_code_id: &Option<u64>,
        decimals: &Option<u8>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::RegisterCw20 {
                    address: address.to_string(),
                    owner: owner.as_ref().map(|x| x.to_string()),
                    whitelist: whitelist
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect::<Vec<String>>()),
                    permissionless_burning: permissionless_burning.to_owned(),
                    cw20_code_id: cw20_code_id.to_owned(),
                    decimals: decimals.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_currency_info(
        &mut self,
        sender: ProjectAccount,
        denom_or_address: &str,
        owner: &Option<ProjectAccount>,
        whitelist: &Option<Vec<Addr>>,
        permissionless_burning: &Option<bool>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateCurrencyInfo {
                    denom_or_address: denom_or_address.to_string(),
                    owner: owner.as_ref().map(|x| x.to_string()),
                    whitelist: whitelist
                        .as_ref()
                        .map(|x| x.iter().map(|y| y.to_string()).collect::<Vec<String>>()),
                    permissionless_burning: permissionless_burning.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_metadata_native(
        &mut self,
        sender: ProjectAccount,
        denom: &str,
        metadata: &Metadata,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateMetadataNative {
                    denom: denom.to_string(),
                    metadata: metadata.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_update_metadata_cw20(
        &mut self,
        sender: ProjectAccount,
        address: &Addr,
        project: &Option<&str>,
        description: &Option<&str>,
        logo: &Option<Logo>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::UpdateMetadataCw20 {
                    address: address.to_string(),
                    project: project.as_ref().map(|x| x.to_string()),
                    description: description.as_ref().map(|x| x.to_string()),
                    logo: logo.to_owned(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_exclude_native(
        &mut self,
        sender: ProjectAccount,
        denom: &str,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::ExcludeNative {
                    denom: denom.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_exclude_cw20(
        &mut self,
        sender: ProjectAccount,
        address: &Addr,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::ExcludeCw20 {
                    address: address.to_string(),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_mint(
        &mut self,
        sender: ProjectAccount,
        denom_or_address: &str,
        amount: u128,
        recipient: &Option<&Addr>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_minter_address(),
                &ExecuteMsg::Mint {
                    denom_or_address: denom_or_address.to_string(),
                    amount: Uint128::new(amount),
                    recipient: recipient.as_ref().map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn minter_try_burn(
        &mut self,
        sender: ProjectAccount,
        token: &Token,
        amount: u128,
    ) -> StdResult<AppResponse> {
        let contract_address = &self.get_minter_address();
        let msg = &ExecuteMsg::Burn {};

        add_token_to_exec_msg(self, sender, contract_address, msg, amount, token)
    }

    #[track_caller]
    fn minter_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_minter_address(), &QueryMsg::Config {})
    }

    #[track_caller]

    fn minter_query_currency_info(&self, denom_or_address: &str) -> StdResult<CurrencyInfo> {
        self.app.wrap().query_wasm_smart(
            self.get_minter_address(),
            &QueryMsg::CurrencyInfo {
                denom_or_address: denom_or_address.to_string(),
            },
        )
    }

    #[track_caller]
    fn minter_query_currency_info_list(
        &self,
        amount: u32,
        start_after: &Option<&str>,
    ) -> StdResult<Vec<CurrencyInfo>> {
        self.app.wrap().query_wasm_smart(
            self.get_minter_address(),
            &QueryMsg::CurrencyInfoList {
                amount,
                start_after: start_after.as_ref().map(|x| x.to_string()),
            },
        )
    }

    #[track_caller]
    fn minter_query_currency_info_list_by_owner(
        &self,
        owner: ProjectAccount,
        amount: u32,
        start_after: &Option<&str>,
    ) -> StdResult<Vec<CurrencyInfo>> {
        self.app.wrap().query_wasm_smart(
            self.get_minter_address(),
            &QueryMsg::CurrencyInfoListByOwner {
                owner: owner.to_string(),
                amount,
                start_after: start_after.as_ref().map(|x| x.to_string()),
            },
        )
    }

    #[track_caller]
    fn minter_query_token_count_list(
        &self,
        amount: u32,
        start_after: &Option<&Addr>,
    ) -> StdResult<Vec<(Addr, u16)>> {
        self.app.wrap().query_wasm_smart(
            self.get_minter_address(),
            &QueryMsg::TokenCountList {
                amount,
                start_after: start_after.as_ref().map(|x| x.to_string()),
            },
        )
    }
}
