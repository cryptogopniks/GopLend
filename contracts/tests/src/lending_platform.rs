use cw_multi_test::Executor;
use goplend_base::{
    assets::{Currency, Funds, Token, TokenUnverified},
    converters::{str_to_dec, u128_to_dec},
    error::ContractError,
    lending_platform::{
        msg::MigrateMsg,
        state::{
            BID_MIN_RATE, BORROW_APR, BORROW_FEE_RATE, DISCOUNT_MAX_RATE, DISCOUNT_MIN_RATE,
            MAX_DEPOSIT_APR,
        },
        types::{
            Balances, BidType, BiddedCollateralItem, Borrower, Collateral, Collection,
            CollectionInfo, LiquidationBid, LiquidationItem, Proposal, ProposalType,
            QueryBorrowersResponseItem, QueryCollateralsResponseItem,
            QueryLiquidationBidsByCollectionAddressListResponseItem, RegularBalances,
            ReserveBalances, TokenItem, Vault,
        },
    },
    market_maker::types::OffersResponse,
    oracle::types::RawPriceItem,
};
use lending_platform::{helpers, math};
use speculoos::assert_that;

use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};

use crate::helpers::{
    lending_platform::LendingPlatformExtension,
    market_maker::MarketMakerExtension,
    oracle::OracleExtension,
    scheduler::SchedulerExtension,
    suite::{
        core::{assert_error, to_string_vec, to_token_item_list, Project, INITIAL_LIQUIDITY},
        types::{GetPrice, ProjectAccount, ProjectCoin, ProjectNft, ProjectToken},
    },
};

pub fn get_targets(project: &Project, borrowers: &[String]) -> StdResult<Vec<LiquidationItem>> {
    let rate_config = project.lending_platform_query_rate_config()?;
    let common_config = project.lending_platform_query_common_config()?;
    let lending_platform_address = &project.get_lending_platform_address();
    let borrower_list =
        helpers::offchain::get_borrowers(&project.app.wrap(), lending_platform_address, borrowers)?;

    // get all collaterals
    let collection_and_collateral_list =
        project.lending_platform_query_collateral_list(9, &None)?;

    // get collection prices
    let block_time = project.oracle_query_block_time()?;
    let prices = helpers::offchain::get_collection_prices(
        &project.app.wrap(),
        &project.get_oracle_address(),
        &borrower_list,
        &collection_and_collateral_list,
    )?;

    // accumulators
    let mut targets: Vec<LiquidationItem> = vec![];

    // iterate over borrowers
    for QueryBorrowersResponseItem {
        address: borrower_address,
        borrower: Borrower {
            accumulated_loan,
            loan,
        },
    } in borrower_list
    {
        // get collaterals owned by borrower
        let collection_and_collateral_list_by_borrower = helpers::get_collaterals_by_borrower(
            &collection_and_collateral_list
                .iter()
                .map(|x| (x.address.clone(), x.collateral.clone()))
                .collect::<Vec<(Addr, Vec<Collateral>)>>(),
            &borrower_address,
        );

        let collateral_value_result =
            helpers::get_collateral_value(&collection_and_collateral_list_by_borrower, &prices);

        // go to next borrower if any price is outdated
        if collateral_value_result.is_err() {
            continue;
        }

        let collateral_value = collateral_value_result.unwrap();

        // check ltv
        let ltv_max = math::calc_ltv_max(
            rate_config.bid_min_rate,
            rate_config.discount_max_rate,
            rate_config.discount_min_rate,
        );

        let ltv = math::calc_conditional_ltv(
            ltv_max,
            rate_config.borrow_apr,
            Uint128::zero(),
            Uint128::zero(),
            accumulated_loan,
            loan.amount,
            Uint128::zero(),
            Uint128::zero(),
            collateral_value,
            block_time,
            loan.creation_date,
        );

        // if ltv is low check next borrower, otherwise liquidate required amount
        // no collateral borrowers will be ignored
        if ltv <= ltv_max {
            continue;
        }

        // find bids containing at least 1 borrower's collateral token
        let mut liquidation_bids: Vec<QueryLiquidationBidsByCollectionAddressListResponseItem> =
            project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

        for (collection_address, collateral) in &collection_and_collateral_list_by_borrower {
            // TODO: update on client
            let price_list = project
                .market_maker_query_offer_prices(collection_address)
                .map(|x| x.price_list)
                .unwrap_or_default();
            if price_list.is_empty() {
                continue;
            }

            let liquidator = project
                .market_maker_query_collection_owner(collection_address)?
                .owner;

            let liquidation_bids_from_marketplace = price_list
                .iter()
                .map(|x| LiquidationBid {
                    amount: *x,
                    bid_type: BidType::CollectionOffer,
                    creation_date: block_time,
                    discount: Decimal::zero(),
                    liquidator: liquidator.clone(),
                    token_id_list: collateral
                        .token_item_list
                        .iter()
                        .map(|x| x.id.to_owned())
                        .collect(),
                })
                .collect();

            liquidation_bids.push(QueryLiquidationBidsByCollectionAddressListResponseItem {
                collection_address: collection_address.to_owned(),
                liquidation_bids: liquidation_bids_from_marketplace,
            });
        }

        let collection_and_bid_list = helpers::offchain::get_collection_and_bid_list(
            &liquidation_bids,
            &collection_and_collateral_list_by_borrower,
        )?;

        let bidded_collateral_list = math::match_collaterals_and_bids(
            &collection_and_collateral_list_by_borrower,
            &collection_and_bid_list,
            &prices,
        )?;

        // calculate how much tokens and what collection must be liquidated to get proper ltv
        // for worst case (max discount)
        let max_liquidation_value = math::calc_liquidation_value(
            collateral_value,
            rate_config.discount_max_rate,
            ltv,
            ltv_max,
        );

        let liquidation_set = math::calc_liquidation_set(
            &bidded_collateral_list,
            max_liquidation_value,
            ltv_max,
            rate_config.borrow_apr,
            accumulated_loan,
            loan.amount,
            collateral_value,
            block_time,
            loan.creation_date,
        );

        targets.push(LiquidationItem {
            borrower: borrower_address.to_string(),
            liquidation_set,
        });
    }

    Ok(targets)
}

#[test]
fn migrate_default() {
    let mut project = Project::new();

    project
        .app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            project.get_lending_platform_address(),
            &MigrateMsg {
                version: "2.0.0".to_string(),
            },
            project.get_lending_platform_code_id(),
        )
        .unwrap();
}

#[test]
fn deposit_apr_with_depositors_and_borrowers() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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

    // add collateral
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // check initial deposit apr and bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    // check deposit apr and bgl price after 1/2 year
    project.wait(DELAY / 2);

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    // borrow and check deposit apr and bgl price
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.2"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    // check deposit apr and bgl price after 1/2 year
    project.wait(DELAY / 2);

    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.2"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1.1"));

    Ok(())
}

#[test]
fn deposit_apr_without_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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

    // add collateral
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

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    // check initial deposit apr and bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec(MAX_DEPOSIT_APR));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1"));

    Ok(())
}

#[test]
fn repay_and_withdraw_collateral() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 100;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(200),
        &None,
    )?;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(100),
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

    // add collateral
    for user in [ProjectAccount::Alice, ProjectAccount::Bob] {
        project.increase_allowances_nft(
            user,
            project.get_lending_platform_address(),
            ProjectNft::Gopniks,
        );
    }

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["1"]),
        }],
    )?;
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    // check accumulated loan
    project.wait(DELAY);

    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;

    // repay all
    project.lending_platform_try_repay(
        ProjectAccount::Bob,
        (borrower.loan.amount + borrower.accumulated_loan).u128(),
        ProjectCoin::Usdc,
    )?;

    project.wait(DELAY);

    project.lending_platform_try_withdraw_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
    )?;

    Ok(())
}

#[test]
fn borrow_without_collateral() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)
        .unwrap_err();
    assert_error(&res, ContractError::ExceededLtv);

    Ok(())
}

#[test]
fn borrow_and_repay_all_after_1_year() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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

    // add collateral
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    // check accumulated loan
    project.wait(DELAY);

    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    assert_that(&borrower.accumulated_loan.u128()).is_equal_to(250_000);

    // repay all
    project.lending_platform_try_repay(ProjectAccount::Bob, 1_250_000, ProjectCoin::Usdc)?;

    // check loans
    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    assert_that(&borrower.loan.amount.u128()).is_equal_to(0);
    assert_that(&borrower.accumulated_loan.u128()).is_equal_to(0);

    // check apr, bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    assert_that(&deposit_apr).is_equal_to(str_to_dec("0"));
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1.2"));

    // check platform revenue
    let revenue = project.lending_platform_query_platform_revenue()?;

    assert_that(
        &revenue
            .iter()
            .find(|x| x.currency.token == ProjectCoin::Usdc.into())
            .unwrap()
            .amount
            .u128(),
    )
    .is_equal_to(50_000);

    Ok(())
}

#[test]
fn multiple_borrow_and_repay() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),
                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),
                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
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
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral
    for user in [ProjectAccount::Bob, ProjectAccount::John] {
        project.increase_allowances_nft(
            user,
            project.get_lending_platform_address(),
            ProjectNft::Gopniks,
        );
        project.increase_allowances_nft(
            user,
            project.get_lending_platform_address(),
            ProjectNft::Pinjeons,
        );
    }

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4", "5", "6"]),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["4", "5", "6"]),
            },
        ],
    )?;
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::John,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: vec!["7", "8", "9"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: vec!["7", "8", "9"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
        ],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;
    project.lending_platform_try_borrow(ProjectAccount::John, AMOUNT / 2)?;

    // wait 1/4 year and borrow again
    project.wait(DELAY / 4);

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT / 2)?;
    project.lending_platform_try_borrow(ProjectAccount::John, AMOUNT)?;

    // wait 1/4 year and check loans
    project.wait(DELAY / 4);

    let borrower_bob = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let borrower_john = project.lending_platform_query_borrower(ProjectAccount::John)?;

    //  0.25 * (1_000_000 / 2 + 500_000 / 4) = 156_250
    assert_that(&borrower_bob.accumulated_loan.u128()).is_equal_to(156_250);
    assert_that(&borrower_bob.loan.amount.u128()).is_equal_to(1_500_000);
    //  0.25 * (500_000 / 2 + 1_000_000 / 4) = 125_000
    assert_that(&borrower_john.accumulated_loan.u128()).is_equal_to(125_000);
    assert_that(&borrower_john.loan.amount.u128()).is_equal_to(1_500_000);

    // repay half
    project.lending_platform_try_repay(
        ProjectAccount::Bob,
        (borrower_bob.accumulated_loan + borrower_bob.loan.amount).u128() / 2,
        ProjectCoin::Usdc,
    )?;
    project.lending_platform_try_repay(
        ProjectAccount::John,
        (borrower_john.accumulated_loan + borrower_john.loan.amount).u128() / 2,
        ProjectCoin::Usdc,
    )?;

    // check loans
    let borrower_bob = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let borrower_john = project.lending_platform_query_borrower(ProjectAccount::John)?;

    //  1_500_000 - (828_125 - 156_250) = 828_125
    assert_that(&borrower_bob.accumulated_loan.u128()).is_equal_to(0);
    assert_that(&borrower_bob.loan.amount.u128()).is_equal_to(828_125);
    //  1_500_000 - (812_500 - 125_000) = 812_500
    assert_that(&borrower_john.accumulated_loan.u128()).is_equal_to(0);
    assert_that(&borrower_john.loan.amount.u128()).is_equal_to(812_500);

    // check apr, bgl price
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    let bgl_to_main_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;

    // 0.2 * (828_125 + 812_500) / 1_000_000 = 0.328125
    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.328125"));
    // (1_000_000 + 0.2 * (1_500_000 / 2 + 1_500_000 / 4)) / 1_000_000 = 1.225
    assert_that(&bgl_to_main_price).is_equal_to(str_to_dec("1.225"));

    // wait 1/4 year and check loans
    project.wait(DELAY / 4);

    let borrower_bob = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let borrower_john = project.lending_platform_query_borrower(ProjectAccount::John)?;

    assert_that(&borrower_bob.accumulated_loan.u128()).is_equal_to(51_758);
    assert_that(&borrower_bob.loan.amount.u128()).is_equal_to(828_125);
    assert_that(&borrower_john.accumulated_loan.u128()).is_equal_to(50_781);
    assert_that(&borrower_john.loan.amount.u128()).is_equal_to(812_500);

    // check platform revenue
    let revenue = project.lending_platform_query_platform_revenue()?;

    assert_that(
        &revenue
            .iter()
            .find(|x| x.currency.token == ProjectCoin::Usdc.into())
            .unwrap()
            .amount
            .u128(),
    )
    .is_equal_to(56_250);

    Ok(())
}

#[test]
fn withdraw_from_reserve_and_partial_repay_smaller_than_rewards() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
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

    // add collateral
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    project.wait(DELAY);

    // check balances
    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, AMOUNT, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // check balances
    let balances = project.lending_platform_query_balances()?;
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let stars_difference = stars_after - stars_before;

    assert_that(&stars_difference).is_equal_to(1_200_000);
    // 0.2 * 1_000_000 * ((365 + 14) * 24 * 3600) / (365 * 24 * 3600) = 207_671
    assert_that(&balances.regular.accumulated_rewards.u128()).is_equal_to(207_671);

    // repay
    project.lending_platform_try_repay(ProjectAccount::Bob, AMOUNT / 10, ProjectCoin::Usdc)?;

    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.main_pool.u128()).is_equal_to(80_000);
    assert_that(&balances.regular.accumulated_rewards.u128()).is_equal_to(127_671);

    Ok(())
}

#[test]
fn withdraw_from_reserve_and_partial_repay_greater_than_rewards() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
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

    // add collateral
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    project.wait(DELAY);

    // check balances
    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, AMOUNT, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // check balances
    let balances = project.lending_platform_query_balances()?;
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let stars_difference = stars_after - stars_before;

    assert_that(&stars_difference).is_equal_to(1_200_000);
    // 0.2 * 1_000_000 * ((365 + 14) * 24 * 3600) / (365 * 24 * 3600) = 207_671
    assert_that(&balances.regular.accumulated_rewards.u128()).is_equal_to(207_671);

    // repay
    project.lending_platform_try_repay(ProjectAccount::Bob, 1_207_671, ProjectCoin::Usdc)?;

    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.main_pool.u128()).is_equal_to(207_671);
    assert_that(&balances.regular.accumulated_rewards.u128()).is_equal_to(0);

    assert_that(&balances.regular.borrowed.amount.u128()).is_equal_to(51_918);
    assert_that(&balances.reserve.withdrawn.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn withdraw_from_reserve_and_full_repay() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
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

    // add collateral
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    project.wait(DELAY);

    // check balances
    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, AMOUNT, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // check balances
    let balances = project.lending_platform_query_balances()?;
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let stars_difference = stars_after - stars_before;

    assert_that(&stars_difference).is_equal_to(1_200_000);
    // 0.2 * 1_000_000 * ((365 + 14) * 24 * 3600) / (365 * 24 * 3600) = 207_671
    assert_that(&balances.regular.accumulated_rewards.u128()).is_equal_to(207_671);

    // repay
    project.lending_platform_try_repay(ProjectAccount::Bob, 1_259_589, ProjectCoin::Usdc)?;

    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.main_pool.u128()).is_equal_to(207_671);
    assert_that(&balances.regular.accumulated_rewards.u128()).is_equal_to(0);

    assert_that(&balances.regular.borrowed.amount.u128()).is_equal_to(0);
    assert_that(&balances.reserve.withdrawn.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn withdraw_from_reserve_limit() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000_000_000;
    const AMOUNT: u128 = 1_000_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &Some("0.85"),
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

    // add collateral
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

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // try exceed borrow limit
    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, 86 * AMOUNT + 1)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, 86 * AMOUNT)?;

    project.wait(DELAY);

    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    let bgl_amount = (u128_to_dec(15 * AMOUNT) / bgl_price)
        .to_uint_floor()
        .u128();

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, bgl_amount, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // try exceed limit
    project.lending_platform_try_unbond(ProjectAccount::Alice, 1, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project
        .lending_platform_try_withdraw(ProjectAccount::Alice)
        .unwrap_err();

    Ok(())
}

#[test]
fn borrow_and_withdraw_from_reserve_limit() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000_000_000;
    const AMOUNT: u128 = 1_000_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &Some("0.85"),
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

    // add collateral
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

    // try exceed borrow limit
    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, 86 * AMOUNT + 1)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, 45 * AMOUNT)?;
    project.wait(1);
    project.lending_platform_try_borrow(ProjectAccount::Bob, 40 * AMOUNT)?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    project.wait(DELAY);

    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    let bgl_amount = (u128_to_dec(16 * AMOUNT) / bgl_price)
        .to_uint_floor()
        .u128();

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, bgl_amount, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // try exceed limit
    project.lending_platform_try_unbond(ProjectAccount::Alice, 1, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project
        .lending_platform_try_withdraw(ProjectAccount::Alice)
        .unwrap_err();

    Ok(())
}

#[test]
fn borrow_and_repay_immediately() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 2_000_000;
    const AMOUNT: u128 = 1_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(DELAY),
        &None,
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

    // add collateral
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

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    //repay immediately
    project.lending_platform_try_repay(ProjectAccount::Bob, AMOUNT, ProjectCoin::Usdc)?;

    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;

    assert_that(&borrower.loan.amount.u128()).is_equal_to(0);
    assert_that(&borrower.accumulated_loan.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn repay_withdraw_loop() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const COLLECTION_PRICE_BEFORE: u128 = 40_000_000_000;
    const AMOUNT: u128 = 1_000_000_000;
    const DELAY: u64 = 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100 * DELAY),
        &None,
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &Some(0),
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

    // add collateral
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
            token_id_list: to_string_vec(&["4", "5"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT, ProjectCoin::Usdc)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, 40 * AMOUNT)?;

    for _i in 0..10 {
        project.wait(DELAY);

        let (_, ltv) = project.lending_platform_query_ltv_list(1, &None)?[0];
        assert_that(&ltv.unwrap().to_string().as_str()).is_equal_to("0.625");

        project.lending_platform_try_repay(ProjectAccount::Bob, 10 * AMOUNT, ProjectCoin::Usdc)?;

        let bgl_price_before =
            project.lending_platform_query_bgl_currency_to_main_currency_price()?;
        let bgl_amount = (u128_to_dec(8 * AMOUNT) / bgl_price_before)
            .to_uint_floor()
            .u128();

        // unbond and withdraw
        project.lending_platform_try_unbond(ProjectAccount::Alice, bgl_amount, &bgl_token)?;
        project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

        let bgl_price_after =
            project.lending_platform_query_bgl_currency_to_main_currency_price()?;

        assert_that(
            &(Decimal::one() - bgl_price_before / bgl_price_after < str_to_dec("0.000000001")),
        )
        .is_equal_to(true);
    }

    Ok(())
}

#[test]
fn deposit_improper_main_currency() -> StdResult<()> {
    let mut project = Project::new();

    const DEPOSIT_AMOUNT: u128 = 1_000;

    // deposit improper token
    let res = project
        .lending_platform_try_deposit(ProjectAccount::Alice, DEPOSIT_AMOUNT, ProjectCoin::Stars)
        .unwrap_err();

    assert_error(&res, ContractError::WrongAssetType);

    Ok(())
}

#[test]
fn deposit_zero() -> StdResult<()> {
    let mut project = Project::new();

    const DEPOSIT_AMOUNT: u128 = 0;

    let res = project
        .lending_platform_try_deposit(ProjectAccount::Alice, DEPOSIT_AMOUNT, ProjectCoin::Usdc)
        .unwrap_err();

    assert_error(&res, "Cannot transfer empty coins amount");

    Ok(())
}

#[test]
fn deposit_default() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let bgl_stars_before = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Usdc,
    )?;

    // check user balances
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let bgl_stars_after = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    let stars_difference = stars_before - stars_after;
    let bgl_stars_difference = bgl_stars_after - bgl_stars_before;

    assert_that(&stars_difference).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&bgl_stars_difference).is_equal_to(DEPOSIT_AMOUNT);

    // check contract balances
    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.deposited.u128()).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(DEPOSIT_AMOUNT);
    assert_that(&balances.regular.bgl_supply.u128()).is_equal_to(DEPOSIT_AMOUNT);

    Ok(())
}

#[test]
fn deposit_does_not_change_bgl_price() -> StdResult<()> {
    let mut project = Project::new();

    const DEPOSIT_AMOUNT: u128 = 1_000;

    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Usdc,
    )?;

    // deposit doesn't change bgl token price
    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

    project.lending_platform_try_deposit(ProjectAccount::Bob, DEPOSIT_AMOUNT, ProjectCoin::Usdc)?;

    let bgl_price = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&bgl_price).is_equal_to(str_to_dec("1"));

    Ok(())
}

#[test]
fn collection_address_duplication() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(800_000_000),
        }],
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

    // collateral value
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[
                CollectionInfo {
                    collection_address: ProjectNft::Gopniks.to_string(),
                    token_id_list: to_string_vec(&["4", "5"]),
                },
                CollectionInfo {
                    collection_address: ProjectNft::Gopniks.to_string(),
                    token_id_list: to_string_vec(&["6"]),
                },
            ],
        )
        .unwrap_err();

    assert_error(&res, ContractError::CollectionDuplication);

    Ok(())
}

#[test]
fn collateral_value_limitations() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(800_000_000),
        }],
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

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1_200_000_000),
        &None,
    )?;

    // collateral value = 800_000_000
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4"]),
            }],
        )
        .unwrap_err();

    assert_error(&res, "min collateral");

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
    )?;

    let res = project
        .lending_platform_try_withdraw_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4"]),
            }],
        )
        .unwrap_err();

    assert_error(&res, "min collateral");

    project.lending_platform_try_withdraw_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
    )?;

    Ok(())
}

#[test]
fn deposit_unbond_withdraw_default() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let bgl_stars_before = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Usdc,
    )?;

    // check contract balances
    let balances = project.lending_platform_query_balances()?;
    assert_that(&balances.regular.deposited.u128()).is_equal_to(DEPOSIT_AMOUNT);

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT, &bgl_token)?;

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    // check user balances
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    let bgl_stars_after = project.query_balance(ProjectAccount::Alice, &bgl_token)?;

    let stars_difference = stars_before - stars_after;
    let bgl_stars_difference = bgl_stars_after - bgl_stars_before;

    assert_that(&stars_difference).is_equal_to(0);
    assert_that(&bgl_stars_difference).is_equal_to(0);

    // check contract balances
    let balances = project.lending_platform_query_balances()?;

    assert_that(&balances.regular.deposited.u128()).is_equal_to(0);
    assert_that(&balances.regular.unbonded.u128()).is_equal_to(0);
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(0);
    assert_that(&balances.regular.bgl_supply.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn withdraw_reserve_liquidity_default() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Usdc,
    )?;

    // check contract balances
    let balances = project.lending_platform_query_balances()?;
    assert_that(&balances.regular.bgl_supply.u128()).is_equal_to(DEPOSIT_AMOUNT);

    let admin_main_balance_before =
        project.query_balance(ProjectAccount::Admin, &ProjectCoin::Usdc)?;

    let admin_bgl_balance = project.query_balance(ProjectAccount::Admin, &bgl_token)?;
    assert_that(&admin_bgl_balance).is_equal_to(0);

    // unbond
    project.lending_platform_try_withdraw_reserve_liquidity(
        ProjectAccount::Admin,
        INITIAL_LIQUIDITY,
    )?;

    let admin_main_balance_after =
        project.query_balance(ProjectAccount::Admin, &ProjectCoin::Usdc)?;

    assert_that(&(admin_main_balance_after - admin_main_balance_before))
        .is_equal_to(INITIAL_LIQUIDITY);

    Ok(())
}

#[test]
fn multiple_unbonding_positions() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Usdc,
    )?;

    let stars_before = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT / 8, &bgl_token)?;
    project.wait(config.unbonding_period / 4);
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT / 4, &bgl_token)?;
    project.wait(config.unbonding_period / 4);
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT / 2, &bgl_token)?;
    let unbonder = project.lending_platform_query_unbonder(ProjectAccount::Alice)?;
    assert_that(&unbonder.returned_and_unbonded.len()).is_equal_to(3);

    project.wait(config.unbonding_period / 2);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    assert_that(&(stars_after - stars_before)).is_equal_to(DEPOSIT_AMOUNT / 8);

    project.wait(config.unbonding_period / 2);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;
    let stars_after = project.query_balance(ProjectAccount::Alice, &ProjectCoin::Usdc)?;
    assert_that(&(stars_after - stars_before)).is_equal_to(DEPOSIT_AMOUNT * 7 / 8);

    Ok(())
}

#[test]
fn min_available_liquidity_limitation_without_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 85_000_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(80_000_000_000),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &Some("0.85"),
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

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    // total_main_currency_pool - total_borrowed_amount - min_available_liquidity
    // 100_000_000_000 - 0 - 0.15 * 100_000_000_000 = 85_000_000_000
    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(AMOUNT);

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT + 1)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn min_available_liquidity_limitation_with_depositors() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 1_000_000;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(80_000_000_000),
        }],
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &Some("0.85"),
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

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Alice, AMOUNT / 2, ProjectCoin::Usdc)?;

    // min_available_liquidity = 0.15 * reserve_deposited
    // 0.15 * 100_000_000_000 = 15_000_000_000
    // regular_available = regular_deposited + acc_rewards - regular_unbonded - regular_borrowed
    // 500_000 + 0 - 0 - 0 = 500_000
    // reserve_available = reserve_deposited - min_available_liquidity - reserve_borrowed
    // 100_000_000_000 - 15_000_000_000 - 0 = 85_000_000_000
    // total_available_liquidity = regular_available + reserve_available
    // 500_000 + 85_000_000_000 = 85_000_500_000
    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(85_000_500_000);

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, 85_000_500_001)
        .unwrap_err();
    assert_error(&res, ContractError::LowAvailableLiquidity);

    project.lending_platform_try_borrow(ProjectAccount::Bob, 85_000_500_000)?;

    let total_available_to_borrow_liquidity =
        project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&total_available_to_borrow_liquidity.u128()).is_equal_to(0);

    Ok(())
}

#[test]
fn clear_unbonders_storage() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const DEPOSIT_AMOUNT: u128 = 1_000;

    // depositunbonding_period
    project.lending_platform_try_deposit(
        ProjectAccount::Alice,
        DEPOSIT_AMOUNT,
        ProjectCoin::Usdc,
    )?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Alice, DEPOSIT_AMOUNT, &bgl_token)?;

    let unbonders = project.lending_platform_query_unbonder_list(9, &None)?;
    assert_that(&unbonders.len()).is_equal_to(1);

    // wait and withdraw
    project.wait(config.unbonding_period);
    project.lending_platform_try_withdraw(ProjectAccount::Alice)?;

    let unbonders = project.lending_platform_query_unbonder_list(9, &None)?;
    assert_that(&unbonders.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_collaterals_storage() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(80_000_000_000),
        }],
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

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    let collaterals = project.lending_platform_query_collateral_list(9, &None)?;
    assert_that(&collaterals.len()).is_equal_to(1);

    project.lending_platform_try_withdraw_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    let collaterals = project.lending_platform_query_collateral_list(9, &None)?;
    assert_that(&collaterals.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_borrowers_storage() -> StdResult<()> {
    let mut project = Project::new();

    const AMOUNT: u128 = 1_700;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(80_000_000_000),
        }],
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

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT)?;

    let borrowers = project.lending_platform_query_borrower_list(9, &None)?;
    assert_that(&borrowers.len()).is_equal_to(1);

    project.lending_platform_try_repay(ProjectAccount::Bob, AMOUNT, ProjectCoin::Usdc)?;

    let borrowers = project.lending_platform_query_borrower_list(9, &None)?;
    assert_that(&borrowers.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn clear_liquidators_and_bids_storage() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(80_000_000_000),
        }],
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

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    let block_time = project.get_block_time();
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["5", "6"]),
        }],
        "0",
        800,
        ProjectCoin::Usdc,
    )?;

    let liquidators = project.lending_platform_query_liquidator_list(9, &None)?;
    let bids =
        project.lending_platform_query_liquidation_bids_by_liquidator_address_list(9, &None)?;

    assert_that(&liquidators.len()).is_equal_to(1);
    assert_that(&bids.len()).is_equal_to(1);

    project.lending_platform_try_remove_bid(
        ProjectAccount::Alice,
        &[ProjectNft::Gopniks.to_string()],
        block_time,
    )?;

    let liquidators = project.lending_platform_query_liquidator_list(9, &None)?;
    let bids =
        project.lending_platform_query_liquidation_bids_by_liquidator_address_list(9, &None)?;

    assert_that(&liquidators.is_empty()).is_equal_to(true);
    assert_that(&bids.is_empty()).is_equal_to(true);

    Ok(())
}

#[test]
fn partial_liquidation_single_liquidator_enough_bids_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 700;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_500;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    assert_that(&deposit_apr).is_equal_to(str_to_dec(MAX_DEPOSIT_APR));

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    assert_that(&deposit_apr).is_equal_to(str_to_dec(BORROW_APR) - str_to_dec(BORROW_FEE_RATE));

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
        "0",
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    let collaterals_before = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_before =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

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

    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    assert_that(&collaterals_before).is_equal_to(collaterals_after);
    assert_that(&liquidation_bids_before).is_equal_to(liquidation_bids_after);

    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.2"));

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    // liquidation reduces deposit apr
    let deposit_apr = project.lending_platform_query_conditional_deposit_apr(&None, &None)?;
    assert_that(&deposit_apr).is_equal_to(str_to_dec("0.121764705882352941"));

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let ltv_max = math::calc_ltv_max(
        str_to_dec(BID_MIN_RATE),
        str_to_dec(DISCOUNT_MAX_RATE),
        str_to_dec(DISCOUNT_MIN_RATE),
    );

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["5", "6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: date,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["5"]),
                amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&(ltv_after <= ltv_max)).is_equal_to(true);
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn partial_liquidation_single_liquidator_enough_bids_not_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 650;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_200;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

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

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["5", "6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["5", "6"]),
                amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.807692307692307692");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn partial_liquidation_single_liquidator_not_enough_bids_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 650;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_400;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
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

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["5", "6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: vec![],
                amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.807692307692307692");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn partial_liquidation_single_liquidator_not_enough_bids_not_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 650;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_200;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
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

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["5", "6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: vec![],
                amount: Uint128::new(LIQUIDATION_BID - COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.807692307692307692");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn full_liquidation_single_liquidator_enough_bids_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 400;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_300;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

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

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![Collateral {
            owner: ProjectAccount::Alice.into(),
            token_item_list: to_token_item_list(&["4", "5", "6"], date + 100),
        }],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: vec![],
                amount: Uint128::new(LIQUIDATION_BID - 3 * COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after).is_equal_to(str_to_dec("0.727272727272727272"));
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn full_liquidation_single_liquidator_enough_bids_not_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 400;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_100;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

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

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4", "5"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["6"]),
                amount: Uint128::new(LIQUIDATION_BID - 2 * COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("2.25");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn full_liquidation_single_liquidator_not_enough_bids_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 400;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_300;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
        "0",
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4", "5"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: vec![],
                amount: Uint128::new(LIQUIDATION_BID - 2 * COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("2.25");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn full_liquidation_single_liquidator_not_enough_bids_not_enough_funds() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 400;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 1_100;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
        "0",
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4", "5"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: vec![],
                amount: Uint128::new(LIQUIDATION_BID - 2 * COLLECTION_PRICE_AFTER),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("2.25");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn single_discount_liquidation() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 650;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 2_000;
    const DISCOUNT: &str = "0.05";

    // 5 % discount
    // need to liquidate 2 * 650 = 1_300
    // liquidator will pay 2 * ceil(650 * 0.95) = 2 * 618 = 1_236
    // ltv = (1_700 - 1_236) / (1_950 - 1_300) = 0.7138

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;
    let platform_revenue = project.lending_platform_query_platform_revenue()?;
    let balances_after = project.lending_platform_query_balances()?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["4", "5"], date + 100),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["6"]),
                amount: Uint128::new(
                    LIQUIDATION_BID
                        - 2 * (u128_to_dec(COLLECTION_PRICE_AFTER)
                            * (Decimal::one() - str_to_dec(DISCOUNT)))
                        .to_uint_ceil()
                        .u128(),
                ),
                discount: str_to_dec(DISCOUNT),
                bid_type: BidType::LiquidationBid,
            }],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.713846153846153846");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);
    // 2 * 650 * 0.95 * 0.05 = 62
    assert_that(&platform_revenue[1].amount.u128()).is_equal_to(62);
    assert_that(&balances_after).is_equal_to(Balances {
        regular: RegularBalances {
            deposited: Uint128::new(AMOUNT_TO_BORROW),
            unbonded: Uint128::new(0),
            borrowed: Vault {
                creation_date: 1571797519,
                amount: Uint128::new(0),
            },
            accumulated_rewards: Uint128::new(0),
            liquidation_bids: Uint128::new(
                LIQUIDATION_BID
                    - 2 * (u128_to_dec(COLLECTION_PRICE_AFTER)
                        * (Decimal::one() - str_to_dec(DISCOUNT)))
                    .to_uint_ceil()
                    .u128(),
            ),
            main_pool: Uint128::new(AMOUNT_TO_BORROW),
            bgl_supply: Uint128::new(AMOUNT_TO_BORROW),
        },
        reserve: ReserveBalances {
            deposited: Uint128::new(INITIAL_LIQUIDITY),
            withdrawn: Uint128::new(0),
            borrowed: Vault {
                creation_date: 1571797519,
                // 1700 - 2 * 650 * 0.95 * 0.95 = 526
                amount: Uint128::new(526),
            },
        },
    });

    Ok(())
}

#[test]
fn multiple_discount_liquidation() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 680;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 2_000;
    const DISCOUNT_0: &str = "0";
    const DISCOUNT_1: &str = "0.2";

    // 20, 0 % discount
    // liquidator will pay ceil(680 * 0.8) + ceil(680) = 1_224
    // ltv = (1_700 - 1_224) / 680 = 0.7

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
        DISCOUNT_1,
        LIQUIDATION_BID / 2,
        ProjectCoin::Usdc,
    )?;
    project.wait(10);

    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["5"]),
        }],
        DISCOUNT_0,
        LIQUIDATION_BID / 2,
        ProjectCoin::Usdc,
    )?;
    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;
    let balances_after = project.lending_platform_query_balances()?;
    let platform_revenue = project.lending_platform_query_platform_revenue()?;

    let expected_collateral = vec![QueryCollateralsResponseItem {
        address: ProjectNft::Gopniks.into(),
        collateral: vec![
            Collateral {
                owner: ProjectAccount::Bob.into(),
                token_item_list: to_token_item_list(&["6"], date),
            },
            Collateral {
                owner: ProjectAccount::Alice.into(),
                token_item_list: to_token_item_list(&["5", "4"], date + 110),
            },
        ],
    }];

    let expected_liquidation_bids_after =
        vec![QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![
                LiquidationBid {
                    creation_date: 1571797419,
                    liquidator: ProjectAccount::Alice.into(),
                    token_id_list: vec![],
                    amount: Uint128::new(
                        LIQUIDATION_BID / 2
                            - (u128_to_dec(COLLECTION_PRICE_AFTER)
                                * (Decimal::one() - str_to_dec(DISCOUNT_1)))
                            .to_uint_ceil()
                            .u128(),
                    ),
                    discount: str_to_dec(DISCOUNT_1),
                    bid_type: BidType::LiquidationBid,
                },
                LiquidationBid {
                    creation_date: 1571797429,
                    liquidator: ProjectAccount::Alice.into(),
                    token_id_list: vec![],
                    amount: Uint128::new(
                        LIQUIDATION_BID / 2
                            - (u128_to_dec(COLLECTION_PRICE_AFTER)
                                * (Decimal::one() - str_to_dec(DISCOUNT_0)))
                            .to_uint_ceil()
                            .u128(),
                    ),
                    discount: str_to_dec(DISCOUNT_0),
                    bid_type: BidType::LiquidationBid,
                },
            ],
        }];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.7");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);
    // 2 * 650 * 0.95 * 0.05 = 62
    assert_that(&platform_revenue[1].amount.u128()).is_equal_to(62);
    assert_that(&balances_after).is_equal_to(Balances {
        regular: RegularBalances {
            deposited: Uint128::new(AMOUNT_TO_BORROW),
            unbonded: Uint128::new(0),
            borrowed: Vault {
                creation_date: 1571797529,
                amount: Uint128::new(0),
            },
            accumulated_rewards: Uint128::new(0),
            liquidation_bids: Uint128::new(
                LIQUIDATION_BID
                    - (u128_to_dec(COLLECTION_PRICE_AFTER)
                        * (Decimal::one() - str_to_dec(DISCOUNT_1)))
                    .to_uint_ceil()
                    .u128()
                    - (u128_to_dec(COLLECTION_PRICE_AFTER)
                        * (Decimal::one() - str_to_dec(DISCOUNT_0)))
                    .to_uint_ceil()
                    .u128(),
            ),
            main_pool: Uint128::new(AMOUNT_TO_BORROW),
            bgl_supply: Uint128::new(AMOUNT_TO_BORROW),
        },
        reserve: ReserveBalances {
            deposited: Uint128::new(INITIAL_LIQUIDITY),
            withdrawn: Uint128::new(0),
            borrowed: Vault {
                creation_date: 1571797529,
                // 1700 - 680 * 0.95 - 680 * 0.8 * 0.95 = 538
                amount: Uint128::new(538),
            },
        },
    });

    Ok(())
}

#[test]
fn multiple_collection_liquidation() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 600;
    const AMOUNT_TO_BORROW: u128 = 3_400;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";

    // 10 % discount
    // liquidator will pay ceil(800 * 0.9) = 720
    // ltv = (3400 - 720) / (4500 - 800) = 0.724
    // ltv = (3400 - 1440) / (4500 - 1600) = 0.675

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100),
        &None,
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
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Pinjeons,
    );

    // collateral value = 3 * collection_price
    let date = project.get_block_time();
    for collection in [ProjectNft::Gopniks, ProjectNft::Pinjeons] {
        project.lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: collection.to_string(),
                token_id_list: to_string_vec(&["4", "5", "6"]),
            }],
        )?;
    }

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4", "5", "6"]),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["4", "5", "6"]),
            },
        ],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;
    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let collaterals_after = project.lending_platform_query_collateral_list(9, &None)?;
    let liquidation_bids_after =
        project.lending_platform_query_liquidation_bids_by_collection_address_list(9, &None)?;

    let expected_collateral = vec![
        QueryCollateralsResponseItem {
            address: ProjectNft::Gopniks.into(),
            collateral: vec![
                Collateral {
                    owner: ProjectAccount::Bob.into(),
                    token_item_list: to_token_item_list(&["5", "6"], date),
                },
                Collateral {
                    owner: ProjectAccount::Alice.into(),
                    token_item_list: to_token_item_list(&["4"], date + 100),
                },
            ],
        },
        QueryCollateralsResponseItem {
            address: ProjectNft::Pinjeons.into(),
            collateral: vec![
                Collateral {
                    owner: ProjectAccount::Bob.into(),
                    token_item_list: to_token_item_list(&["6"], date),
                },
                Collateral {
                    owner: ProjectAccount::Alice.into(),
                    token_item_list: to_token_item_list(&["4", "5"], date + 100),
                },
            ],
        },
    ];

    let expected_liquidation_bids_after = vec![
        QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["5", "6"]),
                amount: Uint128::new(
                    LIQUIDATION_BID
                        - (u128_to_dec(COLLECTION_PRICE_AFTER)
                            * (Decimal::one() - str_to_dec(DISCOUNT)))
                        .to_uint_ceil()
                        .u128(),
                ),
                discount: str_to_dec(DISCOUNT),
                bid_type: BidType::LiquidationBid,
            }],
        },
        QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Pinjeons.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: 1571797419,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["6"]),
                amount: Uint128::new(
                    LIQUIDATION_BID
                        - 2 * (u128_to_dec(COLLECTION_PRICE_BEFORE)
                            * (Decimal::one() - str_to_dec(DISCOUNT)))
                        .to_uint_ceil()
                        .u128(),
                ),
                discount: str_to_dec(DISCOUNT),
                bid_type: BidType::LiquidationBid,
            }],
        },
    ];

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.71");
    assert_that(&collaterals_after).is_equal_to(expected_collateral);
    assert_that(&liquidation_bids_after).is_equal_to(expected_liquidation_bids_after);

    Ok(())
}

#[test]
fn deposit_borrow_liquidate_balances() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 700;
    const COLLECTION_PRICE_AFTER_2: u128 = 650;
    const AMOUNT_TO_BORROW: u128 = 1_700;
    const LIQUIDATION_BID: u128 = 2_000;
    const DISCOUNT: &str = "0.1";

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    )?;

    // list collection
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Pinjeons,
    );

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    let platform_revenue_before = project.lending_platform_query_platform_revenue()?;
    let stars_revenue_before = platform_revenue_before
        .iter()
        .find(|x| x.currency.token == ProjectCoin::Usdc.into());
    let balances_before = project.lending_platform_query_balances()?;

    assert_that(&stars_revenue_before).is_equal_to(None);
    assert_that(&balances_before.regular.main_pool.u128()).is_equal_to(AMOUNT_TO_BORROW);
    assert_that(&balances_before.regular.borrowed.amount.u128()).is_equal_to(AMOUNT_TO_BORROW);
    assert_that(&balances_before.regular.liquidation_bids.u128()).is_equal_to(LIQUIDATION_BID);

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let platform_revenue_after = project.lending_platform_query_platform_revenue()?;
    let stars_revenue_after = platform_revenue_after
        .iter()
        .find(|x| x.currency.token == ProjectCoin::Usdc.into())
        .unwrap()
        .amount
        .u128();
    let balances_after = project.lending_platform_query_balances()?;

    // 2 * (700 * 0.9) * 0.05 = 63
    assert_that(&stars_revenue_after).is_equal_to(63);
    // main pool is unchanged as we don't have rewards
    assert_that(&balances_after.regular.main_pool.u128()).is_equal_to(1_700);
    // 1_700 - 2 * (700 * 0.9) * 0.95 = 503
    assert_that(&balances_after.regular.borrowed.amount.u128()).is_equal_to(503);
    // 2_000 - 2 * (700 * 0.9) = 740
    assert_that(&balances_after.regular.liquidation_bids.u128()).is_equal_to(740);

    // add more collateral and borrow again
    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Pinjeons.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Pinjeons.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Pinjeons.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_AFTER_2),
            },
        ],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    let balances_after = project.lending_platform_query_balances()?;

    // limited by deposit amount
    assert_that(&balances_after.regular.borrowed.amount.u128()).is_equal_to(1_700);
    // 503 + 1_700 - 1_700 = 503
    assert_that(&balances_after.reserve.borrowed.amount.u128()).is_equal_to(503);
    assert_that(&balances_after.regular.main_pool.u128()).is_equal_to(1_700);
    // 740 + 2_000 = 2_740
    assert_that(&balances_after.regular.liquidation_bids.u128()).is_equal_to(2_740);

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let platform_revenue_after = project.lending_platform_query_platform_revenue()?;
    let stars_revenue_after = platform_revenue_after
        .iter()
        .find(|x| x.currency.token == ProjectCoin::Usdc.into())
        .unwrap()
        .amount
        .u128();
    let balances_after = project.lending_platform_query_balances()?;

    // 63 + 2 * (650 * 0.9) * 0.05 = 122
    assert_that(&stars_revenue_after).is_equal_to(122);
    // 1700 - (2 * (650 * 0.9) * 0.95 - 503) = 1_092
    assert_that(&balances_after.regular.borrowed.amount.u128()).is_equal_to(1_092);
    // 503 - min(2 * (650 * 0.9) * 0.95, 503) = 0
    assert_that(&balances_after.reserve.borrowed.amount.u128()).is_equal_to(0);
    // main pool is unchanged as we don't have rewards
    assert_that(&balances_after.regular.main_pool.u128()).is_equal_to(1_700);
    // 2_740 - 2 * (650 * 0.9) = 1_570
    assert_that(&balances_after.regular.liquidation_bids.u128()).is_equal_to(1_570);

    Ok(())
}

#[test]
fn ltv_before_and_after_full_liquidation() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 700;
    const AMOUNT_TO_BORROW: u128 = 570;
    const LIQUIDATION_BID: u128 = 2_000;
    const DISCOUNT: &str = "0.1";

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100),
        &None,
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
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Pinjeons,
    );

    // collateral value = 3 * collection_price
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(100);

    // check ltv
    let ltv_before = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;

    // 570 / 800 = 0.7125
    assert_that(&ltv_before.to_string().as_str()).is_equal_to("0.71250056483066971");

    let available_to_borrow =
        project.lending_platform_query_available_to_borrow(ProjectAccount::Bob, &None)?;
    assert_that(&available_to_borrow.u128()).is_equal_to(11);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // check ltv
    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;

    // 570 / 700 = 0.814286359806479669
    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.814286359806479669");

    let available_to_borrow =
        project.lending_platform_query_available_to_borrow(ProjectAccount::Bob, &None)?;
    assert_that(&available_to_borrow.u128()).is_equal_to(0);

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    // check results
    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let ltv_list_after = project.lending_platform_query_ltv_list(9, &None)?;

    let available_to_borrow =
        project.lending_platform_query_available_to_borrow(ProjectAccount::Bob, &None)?;
    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let collateral = project.lending_platform_query_collateral(ProjectNft::Gopniks)?;
    let collateral_by_owner =
        project.lending_platform_query_collateral_by_owner(ProjectAccount::Bob)?;

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0");
    // borrower was removed from storage
    assert_that(&ltv_list_after).is_equal_to(vec![]);
    assert_that(&available_to_borrow.u128()).is_equal_to(0);
    // borrower was removed from storage
    assert_that(&borrower).is_equal_to(Borrower::default());

    assert_that(&collateral).is_equal_to(vec![Collateral {
        owner: ProjectAccount::Alice.into(),
        token_item_list: to_token_item_list(&["4"], date + 100),
    }]);

    assert_that(&collateral_by_owner).is_equal_to(vec![]);

    Ok(())
}

#[test]
fn authorization() -> StdResult<()> {
    let mut project = Project::new();

    // set collateral min value
    let res = project
        .lending_platform_try_update_common_config(
            ProjectAccount::Admin,
            &None,
            &None,
            &None,
            &None,
            &None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::NoParameters);

    let res = project
        .lending_platform_try_update_common_config(
            ProjectAccount::Alice,
            &None,
            &None,
            &None,
            &Some(1),
            &None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::Unauthorized);

    let res = project
        .lending_platform_try_update_common_config(
            ProjectAccount::Owner,
            &None,
            &None,
            &Some(1),
            &None,
            &None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::Unauthorized);

    let res = project
        .lending_platform_try_update_common_config(
            ProjectAccount::Alice,
            &None,
            &None,
            &None,
            &None,
            &None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::NoParameters);

    // transfer admin
    project.lending_platform_try_update_address_config(
        ProjectAccount::Admin,
        &Some(ProjectAccount::Alice),
        &None,
        &None,
        &None,
        &None,
        &None,
    )?;

    project.wait(2 * 7 * 24 * 3600);

    let res = project
        .lending_platform_try_accept_admin_role(ProjectAccount::Alice)
        .unwrap_err();
    assert_error(&res, ContractError::TransferAdminDeadline);

    project.lending_platform_try_update_address_config(
        ProjectAccount::Admin,
        &Some(ProjectAccount::Alice),
        &None,
        &None,
        &None,
        &None,
        &None,
    )?;

    project.lending_platform_try_accept_admin_role(ProjectAccount::Alice)?;

    project.lending_platform_try_update_address_config(
        ProjectAccount::Alice,
        &None,
        &Some(ProjectAccount::Alice),
        &None,
        &None,
        &None,
        &None,
    )?;

    Ok(())
}

#[test]
fn big_numbers() -> StdResult<()> {
    let mut project = Project::new();
    let config = project.lending_platform_query_common_config()?;
    let bgl_token = config
        .bgl_currency
        .ok_or(ContractError::AssetIsNotFound)?
        .token;

    const AMOUNT: u128 = 100_000_000_000_000_000;
    const COLLECTION_PRICE_BEFORE: u128 = 100_000_000_000_000_000;
    const COLLECTION_PRICE_AFTER: u128 = 90_000_000_000_000_000;
    const AMOUNT_TO_BORROW: u128 = 60_000_000_000_000_000;
    const LIQUIDATION_BID: u128 = 100_000_000_000_000_000;
    const DISCOUNT: &str = "0.1";
    const DELAY: u64 = 10 * 365 * 24 * 3600;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(10 * DELAY),
        &None,
    )?;

    // set collateral min value
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &Some(DELAY),
        &Some(AMOUNT),
        &None,
    )?;

    // add liquidity
    project.lending_platform_try_deposit_reserve_liquidity(
        ProjectAccount::Admin,
        AMOUNT,
        ProjectCoin::Usdc,
    )?;

    // list collection
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            AMOUNT,
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            AMOUNT,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        AMOUNT,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        AMOUNT,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Pinjeons,
    );

    // collateral value = 3 * collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(ProjectAccount::Admin, AMOUNT, ProjectCoin::Usdc)?;

    // unbond
    project.lending_platform_try_unbond(ProjectAccount::Admin, AMOUNT / 2, &bgl_token)?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(DELAY);

    let res = project.lending_platform_query_platform_revenue()?;
    assert_that(&res).is_equal_to(vec![Funds {
        amount: Uint128::new(2 * AMOUNT),
        currency: Currency::new(&Token::new_cw20(&ProjectToken::Atom.into()), 6),
    }]);

    let res = project.lending_platform_query_balances()?;
    assert_that(&res).is_equal_to(Balances {
        regular: RegularBalances {
            deposited: Uint128::new(AMOUNT),
            unbonded: Uint128::new(AMOUNT / 2),
            borrowed: Vault {
                creation_date: 1887157419,
                amount: Uint128::new(AMOUNT / 2),
            },
            accumulated_rewards: Uint128::new(AMOUNT * 12 / 10),
            liquidation_bids: Uint128::new(AMOUNT),
            main_pool: Uint128::new(AMOUNT),
            bgl_supply: Uint128::new(AMOUNT),
        },
        reserve: ReserveBalances {
            deposited: Uint128::new(100_000_100_000_000_000),
            withdrawn: Uint128::new(0),
            borrowed: Vault {
                creation_date: 1887157419,
                amount: Uint128::new(AMOUNT / 10),
            },
        },
    });

    let res = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    assert_that(&res).is_equal_to(str_to_dec("2.2"));

    let res = project.lending_platform_query_conditional_deposit_apr(&Some(AMOUNT), &None)?;
    assert_that(&res).is_equal_to(str_to_dec("0.08"));

    let res = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &Some(AMOUNT),
        &None,
        &Some(2 * AMOUNT),
        &None,
    )?;
    assert_that(&res).is_equal_to(str_to_dec("1.025"));

    let res = project.lending_platform_query_total_available_to_borrow_liquidity()?;
    assert_that(&res).is_equal_to(Uint128::new(170_000_060_000_000_000));

    let res = project.lending_platform_query_available_to_borrow(ProjectAccount::Bob, &None)?;
    assert_that(&res).is_equal_to(Uint128::new(8_181_818_181_818_181));

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let _res = project.lending_platform_query_platform_revenue()?;
    let _res = project.lending_platform_query_balances()?;
    let _res = project.lending_platform_query_bgl_currency_to_main_currency_price()?;
    let _res = project.lending_platform_query_conditional_deposit_apr(&Some(AMOUNT), &None)?;
    let _res = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &Some(AMOUNT),
        &None,
        &Some(2 * AMOUNT),
        &None,
    )?;
    let _res = project.lending_platform_query_total_available_to_borrow_liquidity()?;
    let _res = project.lending_platform_query_available_to_borrow(ProjectAccount::Bob, &None)?;

    Ok(())
}

#[test]
fn place_bid_own_collateral() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const LIQUIDATION_BID: u128 = 1_500_000;

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
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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
    project.increase_allowances_nft(
        ProjectAccount::Alice,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
    )?;

    project.transfer_nft(
        ProjectAccount::Bob,
        ProjectAccount::Alice,
        ProjectNft::Gopniks,
        "5",
    );

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["5"]),
        }],
    )?;

    let res = project
        .lending_platform_try_place_bid(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["5"]),
            }],
            "0",
            LIQUIDATION_BID,
            ProjectCoin::Usdc,
        )
        .unwrap_err();
    assert_error(&res, ContractError::OwnCollateralBids);

    let res = project
        .lending_platform_try_place_bid(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["6"]),
            }],
            "0",
            LIQUIDATION_BID,
            ProjectCoin::Usdc,
        )
        .unwrap_err();
    assert_error(&res, ContractError::NftIsNotFound);

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

    let bids = project
        .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;
    assert_that(&bids[0].token_id_list).is_equal_to(
        vec!["4"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    );

    Ok(())
}

#[test]
fn update_bid_replace_tokens() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 700;
    const AMOUNT_TO_BORROW: u128 = 1100;
    const LIQUIDATION_BID: u128 = 1_500_000;

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
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
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

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
    )?;

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

    let bids = project
        .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;

    let res = project
        .lending_platform_try_update_bid(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["4", "5", "6"]),
            }],
            bids[0].creation_date,
            LIQUIDATION_BID,
            "0",
            ProjectCoin::Usdc,
        )
        .unwrap_err();
    assert_error(&res, ContractError::NftIsNotFound);

    let res = project
        .lending_platform_try_update_bid(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: vec!["4", "5", "4"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            }],
            bids[0].creation_date,
            LIQUIDATION_BID,
            "0",
            ProjectCoin::Usdc,
        )
        .unwrap_err();
    assert_error(&res, ContractError::NftDuplication);

    project.lending_platform_try_update_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
        bids[0].creation_date,
        LIQUIDATION_BID,
        "0",
        ProjectCoin::Usdc,
    )?;

    let bids = project
        .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;
    assert_that(&bids[0].token_id_list).is_equal_to(to_string_vec(&["4", "5"]));

    project.lending_platform_try_update_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["5"]),
        }],
        bids[0].creation_date,
        LIQUIDATION_BID,
        "0",
        ProjectCoin::Usdc,
    )?;

    let bids = project
        .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;
    assert_that(&bids[0].token_id_list).is_equal_to(
        vec!["5"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    );

    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    project.wait(CLOCK_PERIOD);

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // process without liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    let bids = project
        .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;
    assert_that(&bids[0].token_id_list).is_equal_to(vec![]);

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["6"]),
        }],
    )?;

    project.lending_platform_try_update_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["6"]),
        }],
        bids[0].creation_date,
        LIQUIDATION_BID,
        "0",
        ProjectCoin::Usdc,
    )?;

    let bids = project
        .lending_platform_query_liquidation_bids_by_collection_address(ProjectNft::Gopniks)?;
    assert_that(&bids[0].token_id_list).is_equal_to(
        vec!["6"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    );

    Ok(())
}

#[test]
fn market_maker_offer_liquidation() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 700;
    const AMOUNT_TO_BORROW: u128 = 570;

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
        }],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100),
        &None,
    )?;

    // set collateral min value and enable marketplace interface
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
    )?;

    // list collection
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // prepare market maker
    project.market_maker_try_set_collection(
        ProjectAccount::Admin,
        ProjectNft::Gopniks,
        ProjectAccount::Owner,
    )?;

    project.market_maker_try_deposit_liquidity(
        ProjectAccount::Owner,
        ProjectNft::Gopniks,
        30 * COLLECTION_PRICE_BEFORE,
        ProjectCoin::Usdc,
    )?;

    project.market_maker_try_update_offers(
        ProjectAccount::Scheduler,
        ProjectNft::Gopniks,
        &[(0, 700), (0, 710)],
    )?;

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Pinjeons,
    );

    // collateral value = collection_price
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    project.wait(100);

    // check ltv
    let ltv_before = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;

    // 570 / 800 = 0.7125
    assert_that(&ltv_before.to_string().as_str()).is_equal_to("0.71250056483066971");

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // check ltv
    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;

    // 570 / 700 = 0.814286359806479669
    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.814286359806479669");

    // check marketplace offers
    let OffersResponse { price_list, .. } =
        project.market_maker_query_offer_prices(ProjectNft::Gopniks)?;
    assert_that(&price_list.len()).is_equal_to(2);

    // check balances
    let balances = project.lending_platform_query_balances()?;
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(AMOUNT_TO_BORROW);
    assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(0);

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    // check results
    let ltv_after = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;
    let ltv_list_after = project.lending_platform_query_ltv_list(9, &None)?;

    let borrower = project.lending_platform_query_borrower(ProjectAccount::Bob)?;
    let collateral = project.lending_platform_query_collateral(ProjectNft::Gopniks)?;
    let collateral_by_owner =
        project.lending_platform_query_collateral_by_owner(ProjectAccount::Bob)?;

    let OffersResponse { price_list, .. } =
        project.market_maker_query_offer_prices(ProjectNft::Gopniks)?;

    // check balances
    let balances = project.lending_platform_query_balances()?;

    assert_that(&ltv_after.to_string().as_str()).is_equal_to("0");
    // borrower was removed from storage
    assert_that(&ltv_list_after).is_equal_to(vec![]);
    // borrower was removed from storage
    assert_that(&borrower).is_equal_to(Borrower::default());

    // collateral is completely removed as it was sent to marketplace bidder
    assert_that(&collateral).is_equal_to(vec![]);
    assert_that(&collateral_by_owner).is_equal_to(vec![]);
    assert_that(&price_list.len()).is_equal_to(1);

    // 570 + (700 - 570 - 0.05 * 700) = 665
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(665);
    assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(0);

    let res = project.query_all_nft(project.get_market_maker_address());
    let (_collection, token_info) = res
        .iter()
        .find(|(collection, _token_info)| collection.to_string() == ProjectNft::Gopniks.to_string())
        .ok_or(ContractError::CollectionIsNotFound)?;
    assert_that(&token_info.tokens.contains(&"4".to_string())).is_equal_to(true);

    Ok(())
}

#[test]
fn regular_bid_and_market_maker_offer_liquidation() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 700;
    const AMOUNT_TO_BORROW: u128 = 500;
    const LIQUIDATION_BID: u128 = 2_000;
    const DISCOUNT: &str = "0.1";

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100_000_000),
        &None,
    )?;

    // set collateral min value and enable marketplace interface
    project.lending_platform_try_update_common_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &Some(1),
        &None,
    )?;

    // list collection
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // prepare market maker
    for collection in [ProjectNft::Gopniks, ProjectNft::Pinjeons] {
        project.market_maker_try_set_collection(
            ProjectAccount::Admin,
            collection,
            ProjectAccount::Owner,
        )?;

        project.market_maker_try_deposit_liquidity(
            ProjectAccount::Owner,
            collection,
            30 * COLLECTION_PRICE_BEFORE,
            ProjectCoin::Usdc,
        )?;

        if matches!(collection, ProjectNft::Gopniks) {
            project.market_maker_try_update_offers(
                ProjectAccount::Scheduler,
                collection,
                &[(0, 700), (0, 710)],
            )?;
        } else {
            project.market_maker_try_update_offers(
                ProjectAccount::Scheduler,
                collection,
                &[(0, 700), (0, 710)],
            )?;
        }
    }

    // add collateral and borrow
    project.increase_allowances_nft(
        ProjectAccount::Alice,
        project.get_lending_platform_address(),
        ProjectNft::Gopniks,
    );
    project.increase_allowances_nft(
        ProjectAccount::Bob,
        project.get_lending_platform_address(),
        ProjectNft::Pinjeons,
    );

    // collateral value = collection_price
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["1"]),
        }],
    )?;
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Pinjeons.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
    )?;

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        2 * AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // borrow
    project.lending_platform_try_borrow(ProjectAccount::Alice, AMOUNT_TO_BORROW)?;
    project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Kate,
        &[CollectionInfo {
            collection_address: ProjectNft::Pinjeons.to_string(),
            token_id_list: to_string_vec(&["4"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    project.wait(2_700_000);

    // check ltv
    let ltv_before_alice = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Alice,
        &None,
        &None,
        &None,
        &None,
    )?;
    let ltv_before_bob = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;

    assert_that(&ltv_before_alice.to_string().as_str()).is_equal_to("0.72957436399217221");
    assert_that(&ltv_before_bob.to_string().as_str()).is_equal_to("0.72957436399217221");

    // check marketplace offers
    let OffersResponse {
        price_list: price_list_gopniks,
        ..
    } = project.market_maker_query_offer_prices(ProjectNft::Gopniks)?;
    let OffersResponse {
        price_list: price_list_pingeons,
        ..
    } = project.market_maker_query_offer_prices(ProjectNft::Pinjeons)?;

    assert_that(&price_list_gopniks.len()).is_equal_to(2);
    assert_that(&price_list_pingeons.len()).is_equal_to(2);

    // check balances
    let balances = project.lending_platform_query_balances()?;
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(2 * AMOUNT_TO_BORROW);
    assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(LIQUIDATION_BID);

    // process with liquidations
    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    // check results
    let ltv_after_alice = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Alice,
        &None,
        &None,
        &None,
        &None,
    )?;
    let ltv_after_bob = project.lending_platform_query_conditional_ltv(
        ProjectAccount::Bob,
        &None,
        &None,
        &None,
        &None,
    )?;

    let collateral_gopniks = project.lending_platform_query_collateral(ProjectNft::Gopniks)?;
    let collateral_pingeons = project.lending_platform_query_collateral(ProjectNft::Pinjeons)?;

    let OffersResponse {
        price_list: price_list_gopniks,
        ..
    } = project.market_maker_query_offer_prices(ProjectNft::Gopniks)?;
    let OffersResponse {
        price_list: price_list_pingeons,
        ..
    } = project.market_maker_query_offer_prices(ProjectNft::Pinjeons)?;

    // check balances
    let balances = project.lending_platform_query_balances()?;

    assert_that(&ltv_after_alice.to_string().as_str()).is_equal_to("0");
    assert_that(&ltv_after_bob.to_string().as_str()).is_equal_to("0");

    // collateral is completely removed as it was sent to marketplace bidder
    assert_that(&collateral_pingeons).is_equal_to(vec![Collateral {
        owner: ProjectAccount::Kate.into(),
        token_item_list: to_token_item_list(&["4"], date + 2_700_000),
    }]);
    assert_that(&collateral_gopniks).is_equal_to(vec![]);

    assert_that(&price_list_gopniks.len()).is_equal_to(1);
    assert_that(&price_list_pingeons.len()).is_equal_to(2);

    // main_currency_pool_addition = 0.95 * 700 + 0.95 * 0.9 * 700 = 1263
    // main_pool = 1000 + 17 + (1263 - 17 - 1000) = 1263
    assert_that(&balances.regular.main_pool.u128()).is_equal_to(1263);
    assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(1370);

    let res = project.query_all_nft(project.get_market_maker_address());
    let (_collection, token_info) = res
        .iter()
        .find(|(collection, _token_info)| collection.to_string() == ProjectNft::Gopniks.to_string())
        .ok_or(ContractError::CollectionIsNotFound)?;
    assert_that(&token_info.tokens.contains(&"1".to_string())).is_equal_to(true);

    Ok(())
}

// #[test]
// fn liquidation_single_borrower_bid_and_offers() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 900;
//     const COLLECTION_PRICE_AFTER: u128 = 700;
//     const AMOUNT_TO_BORROW: u128 = 1_900;
//     const LIQUIDATION_BID: u128 = 5_000;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection: ProjectNft::Gopniks.to_string(),

//             usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
//         }],
//     )?;

//     project.oracle_try_update_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &None,
//         &Some(100_000_000),
//         &None,
//         &None,
//     )?;

//     // set collateral min value and enable marketplace interface
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &Some(true),
//     )?;

//     // list collection
//     let proposal: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal)?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         AMOUNT_TO_BORROW,
//         ProjectCoin::Usdc,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );

//     // collateral value = 3 * collection_price
//     let date = project.get_block_time();
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection: ProjectNft::Gopniks.to_string(),
//             token_id_list: to_string_vec(&["4", "5", "6"]),
//         }],
//     )?;

//     project.lending_platform_try_borrow(ProjectAccount::Bob, AMOUNT_TO_BORROW)?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection: ProjectNft::Gopniks.to_string(),
//             token_id_list: to_string_vec(&["4"]),
//         }],
//         "0",
//         LIQUIDATION_BID,
//         ProjectCoin::Usdc,
//     )?;

//     project.wait(1);

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection: ProjectNft::Gopniks.to_string(),

//             usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
//         }],
//     )?;

//     // check ltv
//     let ltv = project.lending_platform_query_conditional_ltv(
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;
//     assert_that(&ltv.to_string().as_str()).is_equal_to("0.904761911934357709");

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(9, &None)?
//         .into_iter()
//         .map(|x| x.address.to_string())
//         .collect::<Vec<String>>();
//     let targets = get_targets(&project, &borrowers)?;

//     project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

//     let ltv = project.lending_platform_query_conditional_ltv(
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;

//     let collateral_alice =
//         project.lending_platform_query_collateral_by_owner(ProjectAccount::Alice)?;
//     let collateral_bob = project.lending_platform_query_collateral_by_owner(ProjectAccount::Bob)?;

//     let QueryCollectionOffersResponse {
//         collection_bids: offers_gopniks,
//         ..
//     } = &project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Gopniks, 10)?;

//     let platform_balance =
//         project.query_balance(project.get_lending_platform_address(), &ProjectCoin::Usdc)?;

//     let res = project.query_all_nft(ProjectAccount::John);
//     let (_collection, token_info) = res
//         .iter()
//         .find(|(collection, _token_info)| collection.to_string() == ProjectNft::Gopniks.to_string())
//         .ok_or(ContractError::CollectionIsNotFound)?;

//     assert_that(&ltv.to_string().as_str()).is_equal_to("0.714285714285714285");

//     assert_that(&collateral_alice).is_equal_to(vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![Collateral {
//             owner: ProjectAccount::Alice.into(),
//             token_item_list: to_token_item_list(&["4"], date + 1),
//         }],
//     }]);

//     assert_that(&collateral_bob).is_equal_to(vec![QueryCollateralsResponseItem {
//         address: ProjectNft::Gopniks.into(),
//         collateral: vec![Collateral {
//             owner: ProjectAccount::Bob.into(),
//             token_item_list: to_token_item_list(&["6"], date),
//         }],
//     }]);

//     assert_that(&offers_gopniks.len()).is_equal_to(1);
//     assert_that(&platform_balance).is_equal_to(100_000_005_700);
//     assert_that(&token_info.tokens.contains(&"5".to_string())).is_equal_to(true);

//     Ok(())
// }

// #[test]
// fn regular_bid_and_market_maker_offer_liquidation_on_single_user() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 700;
//     const COLLECTION_PRICE_AFTER: u128 = 400;
//     const AMOUNT_TO_BORROW: u128 = 500;
//     const LIQUIDATION_BID: u128 = 2_000;
//     const DISCOUNT: &str = "0.1";

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[
//             RawPriceItem {
//                 collection: ProjectNft::Gopniks.to_string(),

//                 usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
//             },
//             RawPriceItem {
//                 collection: ProjectNft::Pinjeons.to_string(),

//                 usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
//             },
//         ],
//     )?;

//     project.oracle_try_update_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &None,
//         &Some(100_000_000),
//         &None,
//         &None,
//     )?;

//     // set collateral min value and enable marketplace interface
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &Some(true),
//     )?;

//     // list collection
//     let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection: ProjectNft::Pinjeons.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Pinjeons.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
//     project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         2,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Alice,
//         project.get_lending_platform_address(),
//         ProjectNft::Gopniks,
//     );
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_address(),
//         ProjectNft::Pinjeons,
//     );

//     // collateral value = collection_price
//     let date = project.get_block_time();
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Alice,
//         &[CollectionInfo {
//             collection: ProjectNft::Gopniks.to_string(),
//             token_id_list: to_string_vec(&["1"]),
//         }],
//     )?;
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         &[CollectionInfo {
//             collection: ProjectNft::Pinjeons.to_string(),
//             token_id_list: to_string_vec(&["4", "5"]),
//         }],
//     )?;

//     // deposit
//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         2 * AMOUNT_TO_BORROW,
//         ProjectCoin::Usdc,
//     )?;

//     // borrow
//     project.lending_platform_try_borrow(ProjectAccount::Alice, AMOUNT_TO_BORROW)?;
//     project.lending_platform_try_borrow(ProjectAccount::Bob, 2 * AMOUNT_TO_BORROW)?;

//     // place bid
//     project.lending_platform_try_place_bid(
//         ProjectAccount::Kate,
//         &[CollectionInfo {
//             collection: ProjectNft::Pinjeons.to_string(),
//             token_id_list: to_string_vec(&["4"]),
//         }],
//         DISCOUNT,
//         LIQUIDATION_BID,
//         ProjectCoin::Usdc,
//     )?;

//     project.wait(2_600_000);

//     // reduce prices to enforce liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection: ProjectNft::Pinjeons.to_string(),

//             usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
//         }],
//     )?;

//     // check ltv
//     let ltv_before_alice = project.lending_platform_query_conditional_ltv(
//         ProjectAccount::Alice,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;
//     let ltv_before_bob = project.lending_platform_query_conditional_ltv(
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;

//     assert_that(&ltv_before_alice.to_string().as_str()).is_equal_to("0.729008117706747843");
//     assert_that(&ltv_before_bob.to_string().as_str()).is_equal_to("1.275764205986808726");

//     // check marketplace offers
//     let QueryCollectionOffersResponse {
//         collection_bids: offers_gopniks,
//         ..
//     } = &project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Gopniks, 10)?;
//     let QueryCollectionOffersResponse {
//         collection_bids: offers_pingeons,
//         ..
//     } = project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Pinjeons, 10)?;

//     assert_that(&offers_gopniks.len()).is_equal_to(2);
//     assert_that(&offers_pingeons.len()).is_equal_to(2);

//     // check balances
//     let balances = project.lending_platform_query_balances()?;
//     assert_that(&balances.regular.main_pool.u128()).is_equal_to(2 * AMOUNT_TO_BORROW);
//     assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(LIQUIDATION_BID);

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(9, &None)?
//         .into_iter()
//         .map(|x| x.address.to_string())
//         .collect::<Vec<String>>();
//     let targets = get_targets(&project, &borrowers)?;

//     project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

//     // check results
//     let ltv_after_alice = project.lending_platform_query_conditional_ltv(
//         ProjectAccount::Alice,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;
//     let ltv_after_bob = project.lending_platform_query_conditional_ltv(
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;

//     let collateral_gopniks = project.lending_platform_query_collateral(ProjectNft::Gopniks)?;
//     let collateral_pingeons = project.lending_platform_query_collateral(ProjectNft::Pinjeons)?;

//     let QueryCollectionOffersResponse {
//         collection_bids: offers_gopniks,
//         ..
//     } = &project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Gopniks, 10)?;
//     let QueryCollectionOffersResponse {
//         collection_bids: offers_pingeons,
//         ..
//     } = project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Pinjeons, 10)?;

//     // check balances
//     let balances = project.lending_platform_query_balances()?;

//     assert_that(&ltv_after_alice.to_string().as_str()).is_equal_to("0");
//     assert_that(&ltv_after_bob.to_string().as_str()).is_equal_to("0");

//     // collateral is completely removed as it was sent to marketplace bidder
//     assert_that(&collateral_pingeons).is_equal_to(vec![Collateral {
//         owner: ProjectAccount::Kate.into(),
//         token_item_list: to_token_item_list(&["4"], date + 2_600_000),
//     }]);
//     assert_that(&collateral_gopniks).is_equal_to(vec![]);

//     assert_that(&offers_gopniks.len()).is_equal_to(1);
//     assert_that(&offers_pingeons.len()).is_equal_to(1);

//     // main_currency_pool_addition = 0.95 * 700 + 0.95 * 700 + 0.95 * 0.9 * 400 = 1672
//     // platform_revenue = (1 - 0.95) * 700 + (1 - 0.95) * 700 + (1 - 0.95) * 0.9 * 400 = 88
//     // main_pool = 1000 + 88 + (1672 - 88 - 1500) = 1172
//     assert_that(&balances.regular.main_pool.u128()).is_equal_to(1172);
//     assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(1640);

//     let res = project.query_all_nft(ProjectAccount::John);
//     let (_collection, token_info) = res
//         .iter()
//         .find(|(collection, _token_info)| collection.to_string() == ProjectNft::Gopniks.to_string())
//         .ok_or(ContractError::CollectionIsNotFound)?;
//     assert_that(&token_info.tokens.contains(&"1".to_string())).is_equal_to(true);

//     Ok(())
// }

// #[test]
// fn market_maker_offer_liquidation_stablecoin_market() -> StdResult<()> {
//     let mut project = Project::new();

//     const COLLECTION_PRICE_BEFORE: u128 = 80_000;
//     const COLLECTION_PRICE_AFTER: u128 = 70_000;
//     const AMOUNT_TO_BORROW: u128 = 57_000;

//     // set usdc offers
//     let offers = &[
//         stargaze_mocks_marketplace::state::Bid {
//             id: "1".to_string(),
//             bidder: ProjectAccount::John.into(),
//             collection: ProjectNft::Gopniks.into(),
//         },
//         stargaze_mocks_marketplace::state::Bid {
//             id: "2".to_string(),
//             bidder: ProjectAccount::John.into(),
//             collection: ProjectNft::Pinjeons.into(),
//         },
//         stargaze_mocks_marketplace::state::Bid {
//             id: "3".to_string(),
//             bidder: ProjectAccount::Kate.into(),
//             collection: ProjectNft::Gopniks.into(),
//         },
//         stargaze_mocks_marketplace::state::Bid {
//             id: "4".to_string(),
//             bidder: ProjectAccount::Kate.into(),
//             collection: ProjectNft::Pinjeons.into(),
//         },
//     ];

//     project.stargaze_mocks_market_maker_try_set_bids(offers)?;

//     // set collateral min value and enable marketplace interface
//     project.lending_platform_try_update_common_config(
//         ProjectAccount::Admin,
//         PlatformType::Stable,
//         &None,
//         &None,
//         &None,
//         &Some(1),
//         &None,
//         &Some(true),
//     )?;

//     // push prices to oracle
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection: ProjectNft::Gopniks.to_string(),

//             usd_price: (u128_to_dec(COLLECTION_PRICE_BEFORE) * ProjectCoin::Usdc.get_price())
//                 .to_uint_floor(),
//         }],
//     )?;

//     project.oracle_try_update_config(
//         ProjectAccount::Admin,
//         &None,
//         &None,
//         &None,
//         &None,
//         &Some(100),
//         &None,
//         &None,
//     )?;

//     // list collection
//     let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection: ProjectNft::Gopniks.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Gopniks.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
//         proposal_status: None,
//         listing_price: Funds::new(
//             100u128,
//             &Currency::new(
//                 &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
//                 6,
//             ),
//         ),
//         proposal_type: ProposalType::AddCollection {
//             collection: ProjectNft::Pinjeons.to_string(),
//             collection: Collection {
//                 name: ProjectNft::Pinjeons.to_string(),
//                 owner: ProjectAccount::Owner.to_string(),
//             },
//         },
//     };

//     project.lending_platform_try_create_proposal(
//         ProjectAccount::Admin,
//         PlatformType::Stable,
//         proposal_a,
//     )?;
//     project.lending_platform_try_create_proposal(
//         ProjectAccount::Admin,
//         PlatformType::Stable,
//         proposal_b,
//     )?;

//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         PlatformType::Stable,
//         1,
//         100,
//         ProjectToken::Atom,
//     )?;
//     project.lending_platform_try_accept_proposal(
//         ProjectAccount::Owner,
//         PlatformType::Stable,
//         2,
//         100,
//         ProjectToken::Atom,
//     )?;

//     // add collateral and borrow
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_stable_address(),
//         ProjectNft::Gopniks,
//     );
//     project.increase_allowances_nft(
//         ProjectAccount::Bob,
//         project.get_lending_platform_stable_address(),
//         ProjectNft::Pinjeons,
//     );

//     // collateral value = collection_price
//     project.lending_platform_try_deposit_collateral(
//         ProjectAccount::Bob,
//         PlatformType::Stable,
//         &[CollectionInfo {
//             collection: ProjectNft::Gopniks.to_string(),
//             token_id_list: to_string_vec(&["4"]),
//         }],
//     )?;

//     // deposit
//     let amount_to_borrow = (u128_to_dec(AMOUNT_TO_BORROW) * ProjectCoin::Usdc.get_price())
//         .to_uint_floor()
//         .u128();

//     project.lending_platform_try_deposit(
//         ProjectAccount::Admin,
//         PlatformType::Stable,
//         amount_to_borrow,
//         ProjectCoin::Usdc,
//     )?;

//     // borrow
//     project.lending_platform_try_borrow(
//         ProjectAccount::Bob,
//         PlatformType::Stable,
//         amount_to_borrow,
//     )?;

//     project.wait(100);

//     // check ltv
//     let ltv_before = project.lending_platform_query_conditional_ltv(
//         PlatformType::Stable,
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;

//     // 570 / 800 = 0.7125
//     assert_that(&ltv_before.to_string().as_str()).is_equal_to("0.71250056483066971");

//     // reduce prices to trigger liquidation
//     project.oracle_try_update_prices(
//         ProjectAccount::Scheduler,
//         &[RawPriceItem {
//             collection: ProjectNft::Gopniks.to_string(),

//             usd_price: (u128_to_dec(COLLECTION_PRICE_AFTER) * ProjectCoin::Usdc.get_price())
//                 .to_uint_floor(),
//         }],
//     )?;

//     // check ltv
//     let ltv_after = project.lending_platform_query_conditional_ltv(
//         PlatformType::Stable,
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;

//     // 570 / 700 = 0.814286359806479669
//     assert_that(&ltv_after.to_string().as_str()).is_equal_to("0.814286359806479669");

//     // check marketplace offers
//     let QueryCollectionOffersResponse {
//         collection_bids: offers,
//         ..
//     } = &project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Gopniks, 10)?;
//     assert_that(&offers.len()).is_equal_to(2);

//     // check balances
//     let balances = project.lending_platform_query_balances(PlatformType::Stable)?;
//     let platform_balance_before = project.query_balance(
//         project.get_lending_platform_stable_address(),
//         &ProjectCoin::Usdc,
//     )?;
//     assert_that(&balances.regular.main_pool.u128()).is_equal_to(amount_to_borrow);
//     assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(0);

//     // process with liquidations
//     let borrowers = project
//         .lending_platform_query_borrower_list(PlatformType::Stable, &None, &None)?
//         .into_iter()
//         .map(|x| x.address.to_string())
//         .collect::<Vec<String>>();
//     let mut _targets = get_targets(&project, PlatformType::Stable, &borrowers)?;
//     // rewrite targets as we don't have v1
//     // project.adapter_market_maker_stargaze_query_offers_by_collection_address
//     // to get correct targets for tests anymore
//     let targets = vec![LiquidationItem {
//         borrower: ProjectAccount::Bob.to_string(),
//         liquidation_set: vec![BiddedCollateralItem {
//             collection: ProjectNft::Gopniks.into(),
//             token_item: TokenItem {
//                 id: "4".to_string(),
//                 creation_date: 1571797419,
//             },
//             owner: ProjectAccount::Bob.into(),
//             liquidator: ProjectAccount::John.into(),
//             bid_creation_date: 1571797519,
//             liquidation_price: Uint128::new(700),
//             collateral_price: Uint128::new(700),
//             bid_amount: Uint128::new(700),
//             bid_discount: str_to_dec("0.0002"),
//             bid_type: BidType::CollectionOffer,
//             order_id: "1".to_string(),
//         }],
//     }];

//     project.scheduler_try_push(ProjectAccount::Scheduler, &targets, &Route::Neutron)?;

//     // check results
//     let ltv_after = project.lending_platform_query_conditional_ltv(
//         PlatformType::Stable,
//         ProjectAccount::Bob,
//         &None,
//         &None,
//         &None,
//         &None,
//     )?;
//     let ltv_list_after =
//         project.lending_platform_query_ltv_list(PlatformType::Stable, &None, &None)?;

//     let borrower =
//         project.lending_platform_query_borrower(PlatformType::Stable, ProjectAccount::Bob)?;
//     let collateral =
//         project.lending_platform_query_collateral(PlatformType::Stable, ProjectNft::Gopniks)?;
//     let collateral_by_owner = project
//         .lending_platform_query_collateral_by_owner(PlatformType::Stable, ProjectAccount::Bob)?;

//     let QueryCollectionOffersResponse {
//         collection_bids: offers,
//         ..
//     } = &project.adapter_market_maker_stargaze_query_collection_offers(ProjectNft::Gopniks, 10)?;

//     // check balances
//     let balances = project.lending_platform_query_balances(PlatformType::Stable)?;
//     let platform_balance_after = project.query_balance(
//         project.get_lending_platform_stable_address(),
//         &ProjectCoin::Usdc,
//     )?;

//     assert_that(&ltv_after.to_string().as_str()).is_equal_to("0");
//     // borrower was removed from storage
//     assert_that(&ltv_list_after).is_equal_to(vec![]);
//     // borrower was removed from storage
//     assert_that(&borrower).is_equal_to(Borrower::default());

//     // collateral is completely removed as it was sent to marketplace bidder
//     assert_that(&collateral).is_equal_to(vec![]);
//     assert_that(&collateral_by_owner).is_equal_to(vec![]);
//     assert_that(&offers.len()).is_equal_to(1);

//     // 570 + (700 - 570 - 0.05 * 700) = 665
//     assert_that(&balances.regular.main_pool.u128()).is_equal_to(665);
//     assert_that(&balances.regular.liquidation_bids.u128()).is_equal_to(0);
//     assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(665);

//     let res = project.query_all_nft(ProjectAccount::John);
//     let (_collection, token_info) = res
//         .iter()
//         .find(|(collection, _token_info)| collection.to_string() == ProjectNft::Gopniks.to_string())
//         .ok_or(ContractError::CollectionIsNotFound)?;
//     assert_that(&token_info.tokens.contains(&"4".to_string())).is_equal_to(true);

//     Ok(())
// }

#[test]
fn outdated_prices() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 400;
    const AMOUNT_TO_BORROW: u128 = 320;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100),
        &None,
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
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    for user in [
        ProjectAccount::Alice,
        ProjectAccount::Bob,
        ProjectAccount::John,
        ProjectAccount::Ruby,
    ] {
        for collection in [ProjectNft::Gopniks, ProjectNft::Pinjeons] {
            project.increase_allowances_nft(
                user,
                project.get_lending_platform_address(),
                collection,
            );
        }
    }

    // deposit collateral
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["1", "2"]),
        }],
    )?;

    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Pinjeons.to_string(),
            token_id_list: to_string_vec(&["4", "5"]),
        }],
    )?;

    for collection in [ProjectNft::Gopniks, ProjectNft::Pinjeons] {
        project.lending_platform_try_deposit_collateral(
            ProjectAccount::John,
            &[CollectionInfo {
                collection_address: collection.to_string(),
                token_id_list: to_string_vec(&["7", "8"]),
            }],
        )?;
    }

    // borrow
    for user in [
        ProjectAccount::Alice,
        ProjectAccount::Bob,
        ProjectAccount::John,
    ] {
        project.lending_platform_try_borrow(user, AMOUNT_TO_BORROW)?;
    }

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Kate,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["1", "2", "7", "8"]),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["4", "5", "7", "8"]),
            },
        ],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    // wait to get outdated prices
    project.wait(101);

    // reduce prices to trigger liquidation
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // gopniks - updated, pinjeons - outdated
    //
    // collateral - gopniks, deposit gopniks
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["3"]),
        }],
    )?;
    project.lending_platform_try_withdraw_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["1"]),
        }],
    )?;

    // collateral - gopniks, deposit pinjeons
    // don't allow to deposit collateral with outdated price
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Alice,
            &[CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["3"]),
            }],
        )
        .unwrap_err();
    assert_error(&res, "OutdatedPrice");

    project.lending_platform_try_borrow(ProjectAccount::Alice, 100)?;
    project.lending_platform_try_repay(ProjectAccount::Alice, 100, ProjectCoin::Usdc)?;
    project.lending_platform_query_conditional_ltv(
        ProjectAccount::Alice,
        &None,
        &None,
        &None,
        &None,
    )?;

    // collateral - pinjeons, deposit gopniks
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["6"]),
        }],
    )?;

    let res = project
        .lending_platform_try_withdraw_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["6"]),
            }],
        )
        .unwrap_err();
    assert_error(&res, "outdated");

    // collateral - pinjeons, deposit pinjeons
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["6"]),
            }],
        )
        .unwrap_err();
    assert_error(&res, "OutdatedPrice");

    let res = project
        .lending_platform_try_borrow(ProjectAccount::Bob, 100)
        .unwrap_err();
    assert_error(&res, "outdated");

    project.lending_platform_try_repay(ProjectAccount::Bob, 100, ProjectCoin::Usdc)?;

    let res = project
        .lending_platform_query_conditional_ltv(ProjectAccount::Bob, &None, &None, &None, &None)
        .unwrap_err();
    assert_error(&res, "outdated");

    // collateral - gopniks, pinjeons; deposit gopniks
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::John,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["9"]),
        }],
    )?;

    let res = project
        .lending_platform_try_withdraw_collateral(
            ProjectAccount::John,
            &[CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["9"]),
            }],
        )
        .unwrap_err();
    assert_error(&res, "outdated");

    // collateral - gopniks, pinjeons; deposit pinjeons
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::John,
            &[CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["9"]),
            }],
        )
        .unwrap_err();
    assert_error(&res, "OutdatedPrice");

    let res = project
        .lending_platform_try_borrow(ProjectAccount::John, 100)
        .unwrap_err();
    assert_error(&res, "outdated");

    project.lending_platform_try_repay(ProjectAccount::John, 100, ProjectCoin::Usdc)?;

    let res = project
        .lending_platform_query_conditional_ltv(ProjectAccount::John, &None, &None, &None, &None)
        .unwrap_err();
    assert_error(&res, "outdated");

    // collateral - none, deposit pinjeons
    let res = project
        .lending_platform_try_deposit_collateral(
            ProjectAccount::Ruby,
            &[CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["13"]),
            }],
        )
        .unwrap_err();
    assert_error(&res, "outdated");

    // update prices to allow users borrow more tokens
    project.wait(1);
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
            },
        ],
    )?;

    project.lending_platform_try_borrow(ProjectAccount::Alice, 250)?;
    project.lending_platform_try_borrow(ProjectAccount::Bob, 650)?;
    project.lending_platform_try_borrow(ProjectAccount::John, 1200)?;

    // reduce prices to trigger liquidation
    project.wait(1);
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),
                usd_price: Uint128::new(COLLECTION_PRICE_AFTER / 3),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),
                usd_price: Uint128::new(COLLECTION_PRICE_AFTER / 3),
            },
        ],
    )?;

    // wait to get outdated prices
    project.wait(101);
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),
            usd_price: Uint128::new(COLLECTION_PRICE_AFTER / 3),
        }],
    )?;

    // all users have ltv > ltv_max but some of them will be ignored due to outdated prices
    let ltv_list = project.lending_platform_query_ltv_list(9, &None)?;
    assert_that(&ltv_list).is_equal_to(vec![
        (
            ProjectAccount::Alice.into(),
            Some(str_to_dec("2.142858875570776253")),
        ),
        (ProjectAccount::Bob.into(), None),
        (ProjectAccount::John.into(), None),
    ]);

    // process with liquidations
    project.lending_platform_try_place_bid(
        ProjectAccount::Kate,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["3"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    let borrower_alice = project.lending_platform_query_borrower(ProjectAccount::Alice)?;
    assert_that(&borrower_alice.loan.amount.u128()).is_equal_to(570);

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    // alice got liquidation as her ltv was known
    let borrower_alice = project.lending_platform_query_borrower(ProjectAccount::Alice)?;
    assert_that(&borrower_alice.loan.amount.u128()).is_equal_to(330);

    let ltv_list = project.lending_platform_query_ltv_list(9, &None)?;
    assert_that(&ltv_list).is_equal_to(vec![
        (
            ProjectAccount::Alice.into(),
            Some(str_to_dec("0.727272727272727272")),
        ),
        (ProjectAccount::Bob.into(), None),
        (ProjectAccount::John.into(), None),
    ]);

    // update all prices to liquidate other users
    project.wait(1);
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),
                usd_price: Uint128::new(COLLECTION_PRICE_AFTER / 3),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),
                usd_price: Uint128::new(COLLECTION_PRICE_AFTER / 3),
            },
        ],
    )?;

    // process with liquidations
    project.lending_platform_try_place_bid(
        ProjectAccount::Kate,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["6", "9"]),
        }],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    let borrowers = project
        .lending_platform_query_borrower_list(9, &None)?
        .into_iter()
        .map(|x| x.address.to_string())
        .collect::<Vec<String>>();
    let targets = get_targets(&project, &borrowers)?;

    let ltv_list = project.lending_platform_query_ltv_list(9, &None)?;
    assert_that(&ltv_list).is_equal_to(vec![
        (
            ProjectAccount::Alice.into(),
            Some(str_to_dec("0.727272727272727272")),
        ),
        (
            ProjectAccount::Bob.into(),
            Some(str_to_dec("2.180452908217079988")),
        ),
        (
            ProjectAccount::John.into(),
            Some(str_to_dec("2.135340089426381781")),
        ),
    ]);

    project.scheduler_try_push(ProjectAccount::Scheduler, &targets)?;

    // bob and john got liquidation
    let ltv_list = project.lending_platform_query_ltv_list(9, &None)?;
    assert_that(&ltv_list).is_equal_to(vec![
        (
            ProjectAccount::Alice.into(),
            Some(str_to_dec("0.727272727272727272")),
        ),
        (
            ProjectAccount::Bob.into(),
            Some(str_to_dec("0.727272727272727272")),
        ),
        (
            ProjectAccount::John.into(),
            Some(str_to_dec("0.727272727272727272")),
        ),
    ]);

    Ok(())
}

#[test]
fn ltv_list_for_collections_removed_from_oracle() -> StdResult<()> {
    let mut project = Project::new();

    const COLLECTION_PRICE_BEFORE: u128 = 800;
    const COLLECTION_PRICE_AFTER: u128 = 400;
    const AMOUNT_TO_BORROW: u128 = 320;
    const LIQUIDATION_BID: u128 = 8_000;
    const DISCOUNT: &str = "0.1";

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[
            RawPriceItem {
                collection: ProjectNft::Gopniks.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
            RawPriceItem {
                collection: ProjectNft::Pinjeons.to_string(),

                usd_price: Uint128::new(COLLECTION_PRICE_BEFORE),
            },
        ],
    )?;

    project.oracle_try_update_config(
        ProjectAccount::Admin,
        &None,
        &None,
        &None,
        &None,
        &Some(100),
        &None,
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
    let proposal_a: &Proposal<String, TokenUnverified> = &Proposal {
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

    let proposal_b: &Proposal<String, TokenUnverified> = &Proposal {
        proposal_status: None,
        listing_price: Funds::new(
            100u128,
            &Currency::new(
                &TokenUnverified::new_cw20(&ProjectToken::Atom.to_string()),
                6,
            ),
        ),
        proposal_type: ProposalType::AddCollection {
            collection_address: ProjectNft::Pinjeons.to_string(),
            collection: Collection {
                name: ProjectNft::Pinjeons.to_string(),
                owner: ProjectAccount::Owner.to_string(),
            },
        },
    };

    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_a)?;
    project.lending_platform_try_create_proposal(ProjectAccount::Admin, proposal_b)?;

    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        1,
        100,
        ProjectToken::Atom,
    )?;
    project.lending_platform_try_accept_proposal(
        ProjectAccount::Owner,
        2,
        100,
        ProjectToken::Atom,
    )?;

    // add collateral and borrow
    for user in [ProjectAccount::Alice, ProjectAccount::Bob] {
        for collection in [ProjectNft::Gopniks, ProjectNft::Pinjeons] {
            project.increase_allowances_nft(
                user,
                project.get_lending_platform_address(),
                collection,
            );
        }
    }

    // deposit collateral
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["1", "2"]),
        }],
    )?;

    for collection in [ProjectNft::Gopniks, ProjectNft::Pinjeons] {
        project.lending_platform_try_deposit_collateral(
            ProjectAccount::Bob,
            &[CollectionInfo {
                collection_address: collection.to_string(),
                token_id_list: to_string_vec(&["4", "5"]),
            }],
        )?;
    }

    // borrow
    for user in [ProjectAccount::Alice, ProjectAccount::Bob] {
        project.lending_platform_try_borrow(user, AMOUNT_TO_BORROW)?;
    }

    // deposit
    project.lending_platform_try_deposit(
        ProjectAccount::Admin,
        AMOUNT_TO_BORROW,
        ProjectCoin::Usdc,
    )?;

    // place bid
    project.lending_platform_try_place_bid(
        ProjectAccount::Kate,
        &[
            CollectionInfo {
                collection_address: ProjectNft::Gopniks.to_string(),
                token_id_list: to_string_vec(&["1", "2", "4", "5"]),
            },
            CollectionInfo {
                collection_address: ProjectNft::Pinjeons.to_string(),
                token_id_list: to_string_vec(&["4", "5"]),
            },
        ],
        DISCOUNT,
        LIQUIDATION_BID,
        ProjectCoin::Usdc,
    )?;

    // wait to get outdated prices
    project.wait(101);

    // remove pinjeons
    project.oracle_try_set_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(COLLECTION_PRICE_AFTER),
        }],
    )?;

    // owners of removed from oracle collections will be ignored
    let ltv_list = project.lending_platform_query_ltv_list(9, &None)?;
    assert_that(&ltv_list).is_equal_to(vec![
        (
            ProjectAccount::Alice.into(),
            Some(str_to_dec("0.400000320268899036")),
        ),
        (ProjectAccount::Bob.into(), None),
    ]);

    Ok(())
}

#[test]
fn query_liquidation_bids_by_liquidator_address_default() -> StdResult<()> {
    let mut project = Project::new();

    // push prices to oracle
    project.oracle_try_update_prices(
        ProjectAccount::Scheduler,
        &[RawPriceItem {
            collection: ProjectNft::Gopniks.to_string(),

            usd_price: Uint128::new(80_000_000_000),
        }],
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

    // collateral value = 800_000_000 * 3 = 2_400_000_000
    let date = project.get_block_time();
    project.lending_platform_try_deposit_collateral(
        ProjectAccount::Bob,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["4", "5", "6"]),
        }],
    )?;

    project.lending_platform_try_place_bid(
        ProjectAccount::Alice,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["5"]),
        }],
        "0",
        800,
        ProjectCoin::Usdc,
    )?;
    project.lending_platform_try_place_bid(
        ProjectAccount::Kate,
        &[CollectionInfo {
            collection_address: ProjectNft::Gopniks.to_string(),
            token_id_list: to_string_vec(&["6"]),
        }],
        "0",
        800,
        ProjectCoin::Usdc,
    )?;

    let alice_bids = project
        .lending_platform_query_liquidation_bids_by_liquidator_address(ProjectAccount::Alice)?;
    let kate_bids = project
        .lending_platform_query_liquidation_bids_by_liquidator_address(ProjectAccount::Kate)?;

    assert_that(&alice_bids).is_equal_to(vec![
        QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: date,
                liquidator: ProjectAccount::Alice.into(),
                token_id_list: to_string_vec(&["5"]),
                amount: Uint128::new(800),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        },
    ]);
    assert_that(&kate_bids).is_equal_to(vec![
        QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address: ProjectNft::Gopniks.into(),
            liquidation_bids: vec![LiquidationBid {
                creation_date: date,
                liquidator: ProjectAccount::Kate.into(),
                token_id_list: to_string_vec(&["6"]),
                amount: Uint128::new(800),
                discount: Decimal::zero(),
                bid_type: BidType::LiquidationBid,
            }],
        },
    ]);

    Ok(())
}
