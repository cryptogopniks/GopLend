use cw_multi_test::Executor;
use speculoos::assert_that;

use cosmwasm_std::StdResult;

use goplend_base::{
    assets::{Currency, Funds, TokenUnverified},
    lending_platform::types::{Collection, CollectionInfo, Proposal, ProposalType},
    scheduler::{msg::MigrateMsg, types::Log},
};

use crate::{
    helpers::{
        lending_platform::LendingPlatformExtension,
        oracle::OracleExtension,
        scheduler::SchedulerExtension,
        suite::{
            core::{to_string_vec, Project},
            types::{ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
        },
    },
    lending_platform::get_targets,
    oracle::{to_prices_in, DECIMALS_MULTIPLIER},
};

#[test]
fn push_multiple_clock_sources() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: &str = "800.00";
    const AMOUNT_TO_BORROW: u128 = 1_700 * DECIMALS_MULTIPLIER;
    const LIQUIDATION_BID: u128 = 1_500 * DECIMALS_MULTIPLIER;

    const CLOCK_PERIOD: u64 = 1_000;
    const DELAY: u64 = 100;

    project.scheduler_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(vec![
            ProjectAccount::Alice.into(),
            ProjectAccount::Bob.into(),
        ]),
        &Some(CLOCK_PERIOD),
    )?;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Gopniks, COLLECTION_PRICE_BEFORE)]),
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
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
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Gopniks, "700.00")]),
    )?;

    // process without liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project
        .scheduler_try_push(ProjectAccount::Scheduler, &targets)
        .unwrap_err();

    project.wait(CLOCK_PERIOD);
    project.scheduler_try_push(ProjectAccount::Alice, &targets)?;

    project.wait(DELAY);
    // let res = project
    //     .adapter_scheduler_common_try_push(
    //         ProjectAccount::Bob,
    //         ClockSource::Offchain,
    //         &[],
    //         &Route::Empty,
    //     )
    //     .unwrap_err();
    // assert_error(&res, ContractError::EarlyExecution);

    project.wait(CLOCK_PERIOD - DELAY);

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Gopniks, "600.00")]),
    )?;

    // process without liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Bob, &targets)?;

    let log = project.scheduler_query_log()?;
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

    const COLLECTION_PRICE_BEFORE: &str = "800.00";
    const AMOUNT_TO_BORROW: u128 = 550 * DECIMALS_MULTIPLIER;
    const LIQUIDATION_BID: u128 = 1_500 * DECIMALS_MULTIPLIER;

    const CLOCK_PERIOD: u64 = 1_000;

    project.scheduler_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(CLOCK_PERIOD),
    )?;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &to_prices_in(&[(ProjectNft::Gopniks, COLLECTION_PRICE_BEFORE)]),
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
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
            ProjectCoin::Usdc,
        )?;

        project.wait(CLOCK_PERIOD);

        // push prices to oracle
        project.oracle_try_update_prices(
            ProjectAccount::Scheduler,
            &to_prices_in(&[(ProjectNft::Gopniks, "700.00")]),
        )?;

        // process without liquidations
        let borrowers = project
            .lending_platform_query_borrower_list(9, &None)?
            .into_iter()
            .map(|x| x.address.to_string())
            .collect::<Vec<String>>();
        let targets = get_targets(&project, &borrowers)?;

        project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

        project.wait(1);

        // push prices to oracle
        project.oracle_try_update_prices(
            ProjectAccount::Scheduler,
            &to_prices_in(&[(ProjectNft::Gopniks, COLLECTION_PRICE_BEFORE)]),
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

    let log = project.scheduler_query_log()?;
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
            project.get_scheduler_address(),
            &MigrateMsg {
                version: "2.0.0".to_string(),
            },
            project.get_scheduler_code_id(),
        )
        .unwrap();
}
