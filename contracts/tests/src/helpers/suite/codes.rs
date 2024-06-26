use cosmwasm_std::{coin, coins, Addr, Coin, StdResult, Uint128};
use cw_multi_test::{AppResponse, ContractWrapper, Executor};

use serde::Serialize;
use strum::IntoEnumIterator;

use goplend_base::{
    adapters::{self, marketplace::stargaze::types::MarketplaceVersion},
    assets::{Currency, TokenUnverified},
    converters::str_to_dec,
    error::parse_err,
};

use crate::helpers::suite::{
    core::Project,
    types::{GetDecimals, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
};

pub trait WithCodes {
    // store packages
    fn store_cw20_base_code(&mut self) -> u64;
    fn store_cw721_base_code(&mut self) -> u64;
    fn store_stargaze_mocks_marketplace_code(&mut self) -> u64;
    fn store_adapter_dex_stargaze_mocks_code(&mut self) -> u64;

    // store contracts
    fn store_lending_platform_code(&mut self) -> u64;
    fn store_oracle_code(&mut self) -> u64;
    fn store_minter_code(&mut self) -> u64;
    fn store_adapter_scheduler_common_code(&mut self) -> u64;
    fn store_adapter_marketplace_stargaze_code(&mut self) -> u64;
    fn store_adapter_marketplace_kujira_code(&mut self) -> u64;
    // fn store_adapter_scheduler_stargaze_code(&mut self) -> u64;

    // instantiate packages
    fn instantiate_cw20_base_token(&mut self, code_id: u64, project_token: ProjectToken) -> Addr;
    fn instantiate_cw721_base_token(&mut self, code_id: u64, project_nft: ProjectNft) -> Addr;
    fn instantiate_stargaze_mocks_marketplace(
        &mut self,
        stargaze_mocks_marketplace_code_id: u64,
        bids: &[stargaze_mocks_marketplace::state::Bid],
        amount: u128,
        denom: ProjectCoin,
    ) -> Addr;
    fn instantiate_adapter_dex_stargaze_mocks(
        &mut self,
        adapter_dex_stargaze_mocks_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,
        recover_address: &Option<Addr>,
        max_price_impact: &Option<&str>,
        amount: u128,
        denom_list: &[ProjectCoin],
    ) -> Addr;

    // instantiate contracts
    #[allow(clippy::too_many_arguments)]
    fn instantiate_lending_platform(
        &mut self,
        lending_platform_code_id: u64,

        worker: &Option<ProjectAccount>,
        oracle: &Option<Addr>,
        minter: &Option<Addr>,
        adapter_scheduler: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,

        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,

        main_currency: &Currency<TokenUnverified>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        unbonding_period: &Option<u64>,
        collateral_min_value: &Option<u128>,
        borrowers_reserve_fraction_ratio: &Option<&str>,
        is_marketplace_interface_enabled: &Option<bool>,
    ) -> Addr;

    fn instantiate_oracle(
        &mut self,
        oracle_code_id: u64,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        execution_cooldown: &Option<u64>,
        max_price_update_period: &Option<u64>,
    ) -> Addr;

    fn instantiate_minter(
        &mut self,
        minter_code_id: u64,
        worker: &Option<ProjectAccount>,
        cw20_base_code_id: &Option<u64>,
    ) -> Addr;

    fn instantiate_adapter_scheduler_common(
        &mut self,
        adapter_scheduler_common_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        clock_source: &Option<adapters::scheduler::common::types::ClockSource>,
        execution_cooldown: &Option<u64>,
    ) -> Addr;

    fn instantiate_adapter_marketplace_stargaze(
        &mut self,
        adapter_marketplace_stargaze_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        oracle: &Option<Addr>,
        marketplace: &Option<Addr>,
        adapter_dex: &Option<Addr>,
        marketplace_version: &Option<MarketplaceVersion>,
    ) -> Addr;

    fn instantiate_adapter_marketplace_kujira(
        &mut self,
        adapter_marketplace_kujira_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
    ) -> Addr;

    fn migrate_contract(
        &mut self,
        sender: ProjectAccount,
        contract_address: Addr,
        contract_new_code_id: u64,
        migrate_msg: impl Serialize,
    ) -> StdResult<AppResponse>;
}

impl WithCodes for Project {
    // store packages
    fn store_cw20_base_code(&mut self) -> u64 {
        self.app.store_code(Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        )))
    }

    fn store_cw721_base_code(&mut self) -> u64 {
        self.app.store_code(Box::new(ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        )))
    }

    fn store_stargaze_mocks_marketplace_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                stargaze_mocks_marketplace::contract::execute,
                stargaze_mocks_marketplace::contract::instantiate,
                stargaze_mocks_marketplace::contract::query,
            )
            .with_sudo(stargaze_mocks_marketplace::contract::sudo),
        ))
    }

    fn store_adapter_dex_stargaze_mocks_code(&mut self) -> u64 {
        self.app.store_code(Box::new(ContractWrapper::new(
            adapter_dex_stargaze_mocks::contract::execute,
            adapter_dex_stargaze_mocks::contract::instantiate,
            adapter_dex_stargaze_mocks::contract::query,
        )))
    }

    // store contracts
    fn store_lending_platform_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                lending_platform::contract::execute,
                lending_platform::contract::instantiate,
                lending_platform::contract::query,
            )
            // .with_reply(lending_platform::contract::reply)
            .with_migrate(lending_platform::contract::migrate),
        ))
    }

    fn store_oracle_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                oracle::contract::execute,
                oracle::contract::instantiate,
                oracle::contract::query,
            )
            .with_reply(oracle::contract::reply)
            .with_migrate(oracle::contract::migrate),
        ))
    }

    fn store_minter_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                minter::contract::execute,
                minter::contract::instantiate,
                minter::contract::query,
            )
            .with_reply(minter::contract::reply)
            .with_migrate(minter::contract::migrate),
        ))
    }

    fn store_adapter_scheduler_common_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                adapter_scheduler_common::contract::execute,
                adapter_scheduler_common::contract::instantiate,
                adapter_scheduler_common::contract::query,
            )
            .with_reply(adapter_scheduler_common::contract::reply)
            .with_migrate(adapter_scheduler_common::contract::migrate),
        ))
    }

    // fn store_adapter_scheduler_stargaze_code(&mut self) -> u64 {
    //     self.app.store_code(Box::new(
    //         ContractWrapper::new(
    //             adapter_scheduler_stargaze::contract::execute,
    //             adapter_scheduler_stargaze::contract::instantiate,
    //             adapter_scheduler_stargaze::contract::query,
    //         )
    //         .with_reply(adapter_scheduler_stargaze::contract::reply)
    //         .with_migrate(adapter_scheduler_stargaze::contract::migrate)
    //         .with_sudo(adapter_scheduler_stargaze::contract::sudo),
    //     ))
    // }

    fn store_adapter_marketplace_stargaze_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                adapter_marketplace_stargaze::contract::execute,
                adapter_marketplace_stargaze::contract::instantiate,
                adapter_marketplace_stargaze::contract::query,
            )
            .with_reply(adapter_marketplace_stargaze::contract::reply)
            .with_migrate(adapter_marketplace_stargaze::contract::migrate),
        ))
    }

    fn store_adapter_marketplace_kujira_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                adapter_marketplace_kujira::contract::execute,
                adapter_marketplace_kujira::contract::instantiate,
                adapter_marketplace_kujira::contract::query,
            )
            .with_reply(adapter_marketplace_kujira::contract::reply)
            .with_migrate(adapter_marketplace_kujira::contract::migrate),
        ))
    }

    // instantiate packages
    fn instantiate_cw20_base_token(&mut self, code_id: u64, project_token: ProjectToken) -> Addr {
        let token_postfix: u8 = project_token
            .to_string()
            .strip_prefix("contract")
            .unwrap()
            .parse()
            .unwrap();

        let symbol = format!("TKN{}", "N".repeat(token_postfix as usize)); // max 10 tokens

        let initial_balances: Vec<cw20::Cw20Coin> = ProjectAccount::iter()
            .map(|project_account| {
                let amount = project_account.get_initial_funds_amount()
                    * 10u128.pow(project_token.get_decimals() as u32);

                cw20::Cw20Coin {
                    address: project_account.to_string(),
                    amount: Uint128::from(amount),
                }
            })
            .collect();

        self.instantiate_contract(
            code_id,
            &format!("token{}", "n".repeat(token_postfix as usize)),
            &cw20_base::msg::InstantiateMsg {
                name: format!("cw20-base token {}", symbol),
                symbol,
                decimals: project_token.get_decimals(),
                initial_balances,
                mint: None,
                marketing: None,
            },
        )
    }

    fn instantiate_cw721_base_token(&mut self, code_id: u64, project_nft: ProjectNft) -> Addr {
        let token_postfix: u8 = project_nft
            .to_string()
            .strip_prefix("contract")
            .unwrap()
            .parse()
            .unwrap();

        let symbol = format!("NFT{}", "T".repeat(token_postfix as usize)); // max 10 tokens

        self.instantiate_contract(
            code_id,
            &format!("nft{}", "t".repeat(token_postfix as usize)),
            &cw721_base::msg::InstantiateMsg {
                name: format!("cw721-base token {}", symbol),
                symbol,
                minter: ProjectAccount::Owner.to_string(),
            },
        )
    }

    fn instantiate_stargaze_mocks_marketplace(
        &mut self,
        stargaze_mocks_marketplace_code_id: u64,
        bids: &[stargaze_mocks_marketplace::state::Bid],
        amount: u128,
        denom: ProjectCoin,
    ) -> Addr {
        self.app
            .instantiate_contract(
                stargaze_mocks_marketplace_code_id,
                ProjectAccount::Admin.into(),
                &stargaze_mocks_marketplace::msg::InstantiateMsg {
                    bids: bids.to_owned(),
                },
                &coins(amount, denom.to_string()),
                "stargaze_mocks_marketplace",
                Some(ProjectAccount::Admin.to_string()),
            )
            .unwrap()
    }

    fn instantiate_adapter_dex_stargaze_mocks(
        &mut self,
        adapter_dex_stargaze_mocks_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform_stable: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,
        recover_address: &Option<Addr>,
        max_price_impact: &Option<&str>,
        amount: u128,
        denom_list: &[ProjectCoin],
    ) -> Addr {
        self.app
            .instantiate_contract(
                adapter_dex_stargaze_mocks_code_id,
                ProjectAccount::Admin.into(),
                &goplend_base::adapters::dex::stargaze::msg::InstantiateMsg {
                    worker: worker.as_ref().map(|x| x.to_string()),
                    lending_platform_stable: lending_platform_stable
                        .as_ref()
                        .map(|x| x.to_string()),
                    adapter_marketplace: adapter_marketplace.as_ref().map(|x| x.to_string()),
                    recover_address: recover_address.as_ref().map(|x| x.to_string()),
                    max_price_impact: max_price_impact.as_ref().map(|x| str_to_dec(x)),
                },
                &denom_list
                    .into_iter()
                    .map(|x| coin(amount, x.to_string()))
                    .collect::<Vec<Coin>>(),
                "stargaze_mocks_marketplace",
                Some(ProjectAccount::Admin.to_string()),
            )
            .unwrap()
    }

    // instantiate contracts

    fn instantiate_lending_platform(
        &mut self,
        lending_platform_code_id: u64,

        worker: &Option<ProjectAccount>,
        oracle: &Option<Addr>,
        minter: &Option<Addr>,
        adapter_scheduler: &Option<Addr>,
        adapter_marketplace: &Option<Addr>,

        borrow_apr: &Option<&str>,
        borrow_fee_rate: &Option<&str>,
        liquidation_fee_rate: &Option<&str>,
        discount_min_rate: &Option<&str>,
        discount_max_rate: &Option<&str>,
        bid_min_rate: &Option<&str>,

        main_currency: &Currency<TokenUnverified>,
        bgl_currency: &Option<Currency<TokenUnverified>>,
        unbonding_period: &Option<u64>,
        collateral_min_value: &Option<u128>,
        borrowers_reserve_fraction_ratio: &Option<&str>,
        is_marketplace_interface_enabled: &Option<bool>,
    ) -> Addr {
        self.instantiate_contract(
            lending_platform_code_id,
            "lending_platform",
            &goplend_base::lending_platform::msg::InstantiateMsg {
                worker: worker.as_ref().map(|x| x.to_string()),
                oracle: oracle.as_ref().map(|x| x.to_string()),
                minter: minter.as_ref().map(|x| x.to_string()),
                adapter_scheduler: adapter_scheduler.as_ref().map(|x| x.to_string()),
                adapter_marketplace: adapter_marketplace.as_ref().map(|x| x.to_string()),

                borrow_apr: borrow_apr.as_ref().map(|x| str_to_dec(x)),
                borrow_fee_rate: borrow_fee_rate.as_ref().map(|x| str_to_dec(x)),
                liquidation_fee_rate: liquidation_fee_rate.as_ref().map(|x| str_to_dec(x)),
                discount_min_rate: discount_min_rate.as_ref().map(|x| str_to_dec(x)),
                discount_max_rate: discount_max_rate.as_ref().map(|x| str_to_dec(x)),
                bid_min_rate: bid_min_rate.as_ref().map(|x| str_to_dec(x)),

                main_currency: main_currency.to_owned(),
                bgl_currency: bgl_currency.as_ref().map(|x| x.to_owned()),
                unbonding_period: unbonding_period.as_ref().map(|x| x.to_owned()),
                collateral_min_value: collateral_min_value.map(Uint128::new),
                borrowers_reserve_fraction_ratio: borrowers_reserve_fraction_ratio
                    .as_ref()
                    .map(|x| str_to_dec(x)),
                is_marketplace_interface_enabled: is_marketplace_interface_enabled.to_owned(),
            },
        )
    }

    fn instantiate_oracle(
        &mut self,
        oracle_code_id: u64,
        worker: &Option<ProjectAccount>,
        controller: &Option<Vec<ProjectAccount>>,
        execution_cooldown: &Option<u64>,
        max_price_update_period: &Option<u64>,
    ) -> Addr {
        self.instantiate_contract(
            oracle_code_id,
            "oracle",
            &goplend_base::oracle::msg::InstantiateMsg {
                worker: worker.as_ref().map(|x| x.to_string()),
                controller: controller
                    .as_ref()
                    .map(|x| x.iter().map(|y| y.to_string()).collect()),
                execution_cooldown: execution_cooldown.to_owned(),
                max_price_update_period: max_price_update_period.to_owned(),
            },
        )
    }

    fn instantiate_minter(
        &mut self,
        minter_code_id: u64,
        worker: &Option<ProjectAccount>,
        cw20_base_code_id: &Option<u64>,
    ) -> Addr {
        self.instantiate_contract(
            minter_code_id,
            "minter",
            &goplend_base::minter::msg::InstantiateMsg {
                worker: worker.as_ref().map(|x| x.to_string()),
                cw20_base_code_id: cw20_base_code_id.to_owned(),
            },
        )
    }

    fn instantiate_adapter_scheduler_common(
        &mut self,
        adapter_scheduler_common_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        offchain_clock: &Option<Vec<Addr>>,
        clock_source: &Option<adapters::scheduler::common::types::ClockSource>,
        execution_cooldown: &Option<u64>,
    ) -> Addr {
        self.instantiate_contract(
            adapter_scheduler_common_code_id,
            "adapter_scheduler_common",
            &goplend_base::adapters::scheduler::common::msg::InstantiateMsg {
                worker: worker.as_ref().map(|x| x.to_string()),
                lending_platform_main: lending_platform_main.as_ref().map(|x| x.to_string()),
                lending_platform_stable: lending_platform_stable.as_ref().map(|x| x.to_string()),
                offchain_clock: offchain_clock
                    .as_ref()
                    .map(|x| x.iter().map(|y| y.to_string()).collect()),
                clock_source: clock_source.to_owned(),
                execution_cooldown: execution_cooldown.to_owned(),
            },
        )
    }

    fn instantiate_adapter_marketplace_stargaze(
        &mut self,
        adapter_marketplace_stargaze_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform_main: &Option<Addr>,
        lending_platform_stable: &Option<Addr>,
        oracle: &Option<Addr>,
        marketplace: &Option<Addr>,
        adapter_dex: &Option<Addr>,
        marketplace_version: &Option<MarketplaceVersion>,
    ) -> Addr {
        self.instantiate_contract(
            adapter_marketplace_stargaze_code_id,
            "adapter_marketplace_stargaze",
            &goplend_base::adapters::marketplace::stargaze::msg::InstantiateMsg {
                worker: worker.as_ref().map(|x| x.to_string()),
                lending_platform_main: lending_platform_main.as_ref().map(|x| x.to_string()),
                lending_platform_stable: lending_platform_stable.as_ref().map(|x| x.to_string()),
                oracle: oracle.as_ref().map(|x| x.to_string()),
                marketplace: marketplace.as_ref().map(|x| x.to_string()),
                adapter_dex: adapter_dex.as_ref().map(|x| x.to_string()),
                marketplace_version: marketplace_version.to_owned(),
            },
        )
    }

    fn instantiate_adapter_marketplace_kujira(
        &mut self,
        adapter_marketplace_kujira_code_id: u64,
        worker: &Option<ProjectAccount>,
        lending_platform: &Option<Addr>,
    ) -> Addr {
        self.instantiate_contract(
            adapter_marketplace_kujira_code_id,
            "adapter_marketplace_kujira",
            &goplend_base::adapters::marketplace::kujira::msg::InstantiateMsg {
                worker: worker.as_ref().map(|x| x.to_string()),
                lending_platform: lending_platform.as_ref().map(|x| x.to_string()),
            },
        )
    }

    fn migrate_contract(
        &mut self,
        sender: ProjectAccount,
        contract_address: Addr,
        contract_new_code_id: u64,
        migrate_msg: impl Serialize,
    ) -> StdResult<AppResponse> {
        self.app
            .migrate_contract(
                sender.into(),
                contract_address,
                &migrate_msg,
                contract_new_code_id,
            )
            .map_err(parse_err)
    }
}
