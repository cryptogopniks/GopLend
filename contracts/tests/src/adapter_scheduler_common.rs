use cw_multi_test::Executor;
use speculoos::assert_that;

use cosmwasm_std::{StdResult, Uint128};

use goplend_base::{
    adapters::{
        dex::stargaze::types::Route,
        marketplace::stargaze::types::PlatformType,
        scheduler::common::{
            msg::MigrateMsg,
            types::{ClockSource, Log},
        },
    },
    assets::{Currency, Funds, TokenUnverified},
    error::ContractError,
    lending_platform::types::{Collection, CollectionInfo, Proposal, ProposalType},
    oracle::types::RawPriceItem,
};

use crate::{
    helpers::{
        adapters::scheduler::common::AdapterSchedulerCommonExtension,
        lending_platform::LendingPlatformExtension,
        oracle::OracleExtension,
        suite::{
            core::{assert_error, to_string_vec, Project},
            types::{GetDecimals, GetPrice, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
        },
    },
    lending_platform::get_targets,
};

#[test]
fn push_multiple_clock_sources() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_500;

    const CLOCK_PERIOD: u64 = 1_000;
    const DELAY: u64 = 100;

    project.adapter_scheduler_common_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(vec![
            ProjectAccount::Alice.into(),
            ProjectAccount::Bob.into(),
        ]),
        &None,
        &Some(CLOCK_PERIOD),
    )?;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &ProjectCoin::Stars.get_price().to_string(),
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            usd_price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                    ProjectCoin::Usdc.get_decimals(),
                ),
            ),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &None,
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
        "0",
        LIQUIDATION_BID,
        ProjectCoin::Stars,
    )?;

    project.wait(100);

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &ProjectCoin::Stars.get_price().to_string(),
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(700),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            usd_price: Funds::new(
                Uint128::new(700),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                    ProjectCoin::Usdc.get_decimals(),
                ),
            ),
        }],
    )?;

    // process without liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(&None, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, PlatformType::Main, &borrowers)?;

    project
        .adapter_scheduler_common_try_push(
            ProjectAccount::Scheduler,
            ClockSource::Offchain,
            &targets,
            &Route::Empty,
        )
        .unwrap_err();

    project.wait(CLOCK_PERIOD);
    project.adapter_scheduler_common_try_push(
        ProjectAccount::Alice,
        ClockSource::Offchain,
        &targets,
        &Route::Empty,
    )?;

    project.wait(DELAY);
    let res = project
        .adapter_scheduler_common_try_push(
            ProjectAccount::Bob,
            ClockSource::Offchain,
            &[],
            &Route::Empty,
        )
        .unwrap_err();
    assert_error(&res, ContractError::EarlyExecution);

    project.wait(CLOCK_PERIOD - DELAY);

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &ProjectCoin::Stars.get_price().to_string(),
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(600),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            usd_price: Funds::new(
                Uint128::new(600),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                    ProjectCoin::Usdc.get_decimals(),
                ),
            ),
        }],
    )?;

    // process without liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(&None, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, PlatformType::Main, &borrowers)?;

    project.adapter_scheduler_common_try_push(
        ProjectAccount::Bob,
        ClockSource::Offchain,
        &targets,
        &Route::Empty,
    )?;

    let log = project.adapter_scheduler_common_query_log()?;
    assert_that(&log).is_equal_to(vec![
        Log {
            block_time: 1571799519,
            sender: ProjectAccount::Bob.into(),
            counter: 2,
        },
        Log {
            block_time: 1571798519,
            sender: ProjectAccount::Alice.into(),
            counter: 1,
        },
        Log {
            block_time: 1571797419,
            sender: ProjectAccount::Admin.into(),
            counter: 0,
        },
    ]);

    Ok(())
}

#[test]
fn log_zero_cross() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const AMOUNT_TO_BORROW: u128 = 550;
    const LIQUIDATION_BID: u128 = 1_500_000;

    const CLOCK_PERIOD: u64 = 1_000;

    project.adapter_scheduler_common_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(CLOCK_PERIOD),
    )?;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &ProjectCoin::Stars.get_price().to_string(),
        &[RawPriceItem {
            collection_address: ProjectNft::Gopniks.to_string(),
            price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                    ProjectCoin::Stars.get_decimals(),
                ),
            ),
            usd_price: Funds::new(
                Uint128::new(COLLECTION_PRICE_BEFORE),
                &Currency::new(
                    &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                    ProjectCoin::Usdc.get_decimals(),
                ),
            ),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
        &None,
    )?;

    // list collection
    let proposal: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Gopniks.to_string(),
            collection: Collection {
                name: ProjectNft::Gopniks.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    for _i in 0..(u8::MAX as u16 + 2) {
        // collateral value = 3 * collection_price
        project.lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4"]),
            }],
        )?;

        project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

        // place or update bid
        let bids = project
            .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;

        if !bids.is_empty() {
            project.lending_platform_try_remove_bid(
                ProjectAccount::Alice,
                &[ProjectNft::Gopniks.to_string()],
                bids[0].creation_date,
            )?;
        }

        project.lending_platform_try_place_bid(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4"]),
            }],
            "0",
            LIQUIDATION_BID,
            ProjectCoin::Stars,
        )?;

        project.wait(CLOCK_PERIOD);

        // push prices to oracle
        project.oracle_try_update_prices(
            ProjectAccount::Scheduler,
            &ProjectCoin::Stars.get_price().to_string(),
            &[RawPriceItem {
                collection_address: ProjectNft::Gopniks.to_string(),
                price: Funds::new(
                    Uint128::new(700),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                ),
                usd_price: Funds::new(
                    Uint128::new(700),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                        ProjectCoin::Usdc.get_decimals(),
                    ),
                ),
            }],
        )?;

        // process without liquidations
        let borrowers = project
            .lending_platform_query_borrower_list(&None, &None)?
            .into_iter()
            .map(|x| x.address.to_string())
            .collect::<Vec<String>>();
        let targets = get_targets(&project, PlatformType::Main, &borrowers)?;

        project.adapter_scheduler_common_try_push(
            ProjectAccount::Scheduler,
            ClockSource::Offchain,
            &targets,
            &Route::Empty,
        )?;

        project.wait(1);

        // push prices to oracle
        project.oracle_try_update_prices(
            ProjectAccount::Scheduler,
            &ProjectCoin::Stars.get_price().to_string(),
            &[RawPriceItem {
                collection_address: ProjectNft::Gopniks.to_string(),
                price: Funds::new(
                    Uint128::new(COLLECTION_PRICE_BEFORE),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Stars.to_string()),
                        ProjectCoin::Stars.get_decimals(),
                    ),
                ),
                usd_price: Funds::new(
                    Uint128::new(COLLECTION_PRICE_BEFORE),
                    &Currency::new(
                        &TokenUnverified::new_native(&ProjectCoin::Usdc.to_string()),
                        ProjectCoin::Usdc.get_decimals(),
                    ),
                ),
            }],
        )?;

        project.lending_platform_try_withdraw_collateral(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4"]),
            }],
        )?;

        project.transfer_nft(
            ProjectAccount::Alice,
            ProjectAccount::Bob,
            ProjectNft::Gopniks,
            "4",
        );
    }

    let log = project.adapter_scheduler_common_query_log()?;
    assert_that(&log.into_iter().take(4).collect::<Vec<Log>>()).is_equal_to(vec![
        Log {
            block_time: 1572054675,
            sender: ProjectAccount::Scheduler.into(),
            counter: 1,
        },
        Log {
            block_time: 1572053674,
            sender: ProjectAccount::Scheduler.into(),
            counter: 0,
        },
        Log {
            block_time: 1572052673,
            sender: ProjectAccount::Scheduler.into(),
            counter: 255,
        },
        Log {
            block_time: 1572051672,
            sender: ProjectAccount::Scheduler.into(),
            counter: 254,
        },
    ]);

    Ok(())
}

#[test]
fn migrate_default() {
    let mut project = Project::new();

    project
        .app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            project.get_adapter_scheduler_common_address(),
            &MigrateMsg {
                version: "1.0.0".to_string(),
            },
            project.get_adapter_scheduler_common_code_id(),
        )
        .unwrap();
}
