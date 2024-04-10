use std::cmp::max;

use cosmwasm_std::{Addr, Api, Decimal, Deps, StdResult, Storage, Uint128};

use goplend_base::{
    assets::{Currency, Funds, Token},
    error::ContractError,
    lending_platform::{
        state::{
            ADDRESS_CONFIG, BALANCES, BORROWERS, COLLATERALS, IS_LOCKED, LIQUIDATION_BIDS,
            PLATFORM_REVENUE, RATE_CONFIG,
        },
        types::{
            AddressConfig, Balances, BiddedCollateralItem, Borrower, Collateral, CollectionInfo,
            LiquidationBid, Proposal, ProposalStatus, Vault,
        },
    },
    oracle::{self, types::PriceItem},
    utils::{unwrap_field, AuthType},
};

use crate::math;

pub fn check_lockout(deps: Deps) -> StdResult<()> {
    if IS_LOCKED.load(deps.storage)? {
        Err(ContractError::ContractIsLocked)?;
    }

    Ok(())
}

pub fn verify_proposal_status(
    proposal: &Proposal<Addr, Token>,
    expected: ProposalStatus,
) -> StdResult<()> {
    let proposal_status = unwrap_field(proposal.proposal_status.as_ref(), "proposal_status")?;

    if proposal_status != &expected {
        Err(ContractError::WrongProposalStatus)?;
    }

    Ok(())
}

pub fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let AddressConfig { admin, owner, .. } = ADDRESS_CONFIG.load(deps.storage)?;
    let owner = unwrap_field(owner, "owner");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrOwner => {
            if !((sender == admin) || (owner.is_ok() && sender == owner?)) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::Specified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !is_included {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrOwnerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (owner.is_ok() && sender == owner?) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
    };

    Ok(())
}

pub fn check_collections_holder(
    deps: Deps,
    holder: &Addr,
    collections: &Vec<CollectionInfo<String>>,
) -> StdResult<()> {
    const MAX_LIMIT: u32 = 100;
    const ITER_LIMIT: u32 = 50;

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        let mut token_list: Vec<String> = vec![];
        let mut token_amount_sum: u32 = 0;
        let mut i: u32 = 0;
        let mut last_token: Option<String> = None;

        while (i == 0 || token_amount_sum == MAX_LIMIT) && i < ITER_LIMIT {
            i += 1;

            let query_tokens_msg = cw721::Cw721QueryMsg::Tokens {
                owner: holder.to_string(),
                start_after: last_token,
                limit: Some(MAX_LIMIT),
            };

            let cw721::TokensResponse { tokens } = deps
                .querier
                .query_wasm_smart(collection_address, &query_tokens_msg)?;

            for token in tokens.clone() {
                token_list.push(token);
            }

            token_amount_sum = tokens.len() as u32;
            last_token = tokens.last().cloned();
        }

        let are_tokens_owned = token_id_list.iter().all(|x| token_list.contains(x));

        if !are_tokens_owned {
            Err(ContractError::NftIsNotFound)?;
        }
    }

    Ok(())
}

pub fn accumulate_expected_rewards(storage: &mut dyn Storage, block_time: u64) -> StdResult<()> {
    let rate_config = RATE_CONFIG.load(storage)?;

    BALANCES.update(storage, |mut balances| -> StdResult<Balances> {
        let regular_rewards = math::calc_expected_rewards(
            block_time,
            balances.regular.borrowed.creation_date,
            rate_config.borrow_apr,
            rate_config.borrow_fee_rate,
            balances.regular.borrowed.amount,
        );

        let reserve_rewards = math::calc_expected_rewards(
            block_time,
            balances.reserve.borrowed.creation_date,
            rate_config.borrow_apr,
            rate_config.borrow_fee_rate,
            balances.reserve.borrowed.amount,
        );

        balances.regular.accumulated_rewards += regular_rewards + reserve_rewards;
        balances.regular.borrowed.creation_date = block_time;
        balances.reserve.borrowed.creation_date = block_time;

        Ok(balances)
    })?;

    Ok(())
}

pub fn increase_platform_revenue(
    storage: &mut dyn Storage,
    asset_amount: Uint128,
    asset_info: &Token,
    listing_price_currency: &Currency<Token>,
) -> StdResult<()> {
    let mut funds_list = PLATFORM_REVENUE.load(storage)?;

    let funds = funds_list
        .iter()
        .find(|Funds { currency, .. }| &currency.token == asset_info);

    if funds.is_none() {
        funds_list.push(Funds::new(Uint128::zero(), listing_price_currency));
    }

    for funds_list_item in funds_list.iter_mut() {
        if &funds_list_item.currency.token == asset_info {
            funds_list_item.amount += asset_amount;
        }
    }

    PLATFORM_REVENUE.save(storage, &funds_list)?;

    Ok(())
}

// pub fn get_borrowers(
//     storage: &dyn Storage,
//     api: &dyn Api,
//     borrowers: &[String],
// ) -> StdResult<Vec<(Addr, Borrower)>> {
//     let borrowers: Vec<(Addr, Borrower)> = borrowers
//         .iter()
//         .map(|y| {
//             let address = api.addr_validate(y)?;
//             let borrower = BORROWERS.load(storage, &address)?;

//             Ok((address, borrower))
//         })
//         .collect::<StdResult<Vec<(Addr, Borrower)>>>()?;

//     Ok(borrowers)
// }

// pub fn get_collection_prices(
//     deps: Deps,
//     borrowers: &[(Addr, Borrower)],
//     collection_and_collateral_list: &[(Addr, Vec<Collateral>)],
//     oracle: &Addr,
// ) -> StdResult<Vec<oracle::types::PriceItem>> {
//     // collect collection addresses over current page borrowers
//     let borrower_addresses: Vec<Addr> = borrowers
//         .iter()
//         .map(|(address, _)| address.to_owned())
//         .collect();

//     let mut collection_addresses_by_borrowers: Vec<String> = collection_and_collateral_list
//         .iter()
//         .filter(|(_collection_address, collaterals)| {
//             collaterals
//                 .iter()
//                 .any(|collateral| borrower_addresses.contains(&collateral.owner))
//         })
//         .map(|(collection_address, _)| collection_address.to_string())
//         .collect();

//     collection_addresses_by_borrowers.sort_unstable();
//     collection_addresses_by_borrowers.dedup();

//     // query collection prices
//     let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
//         oracle,
//         &oracle::msg::QueryMsg::QueryPrices {
//             collection_addresses: Some(collection_addresses_by_borrowers),
//             start_after: None,
//             limit: None,
//         },
//     )?;

//     Ok(prices)
// }

// pub fn get_collaterals_by_borrower(
//     collection_and_collateral_list: &[(Addr, Vec<Collateral>)],
//     borrower_address: &Addr,
// ) -> StdResult<Vec<(Addr, Collateral)>> {
//     // get collaterals owned by borrower
//     let collection_and_collateral_list_by_borrower: Vec<(Addr, Collateral)> =
//         collection_and_collateral_list
//             .iter()
//             .cloned()
//             .map(
//                 |(collection, collaterals)| -> StdResult<(Addr, Collateral)> {
//                     // single collateral per collection
//                     let collateral = collaterals
//                         .into_iter()
//                         .find(|collateral| collateral.owner == borrower_address)
//                         .ok_or(ContractError::CollateralIsNotFound)?;

//                     Ok((collection, collateral))
//                 },
//             )
//             .collect::<StdResult<Vec<(Addr, Collateral)>>>()?;

//     Ok(collection_and_collateral_list_by_borrower)
// }

// pub fn get_collateral_value(
//     collection_and_collateral_list_by_borrower: &[(Addr, Collateral)],
//     prices: &[PriceItem],
//     block_time: u64,
//     price_update_period: u64,
// ) -> StdResult<Option<Uint128>> {
//     let mut is_price_outdated = false;

//     let collateral_value = collection_and_collateral_list_by_borrower.iter().fold(
//         Ok(Uint128::zero()),
//         |acc, (collection_address, collateral)| -> StdResult<Uint128> {
//             let oracle::types::PriceItem {
//                 price,
//                 price_update_date,
//                 ..
//             } = prices
//                 .iter()
//                 .find(|x| x.address == collection_address)
//                 .ok_or(ContractError::PriceIsNotFound {
//                     val: collection_address.to_string(),
//                 })?;

//             if block_time - price_update_date > price_update_period {
//                 is_price_outdated = true;
//             }

//             // TODO: decimals, other currencies
//             Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
//         },
//     )?;

//     Ok(if is_price_outdated {
//         None
//     } else {
//         Some(collateral_value)
//     })
// }

// // find bids containing at least 1 borrower's collateral token
// pub fn get_collection_and_bid_list(
//     storage: &dyn Storage,
//     collection_and_collateral_list_by_borrower: &Vec<(Addr, Collateral)>,
// ) -> StdResult<Vec<(Addr, LiquidationBid)>> {
//     let mut collection_and_bid_list: Vec<(Addr, LiquidationBid)> = vec![];

//     for (collection_address, collateral) in collection_and_collateral_list_by_borrower {
//         for bid in LIQUIDATION_BIDS.load(storage, collection_address)? {
//             if bid
//                 .token_id_list
//                 .iter()
//                 .any(|id| collateral.token_id_list.contains(id))
//             {
//                 collection_and_bid_list.push((collection_address.clone(), bid));
//             }
//         }
//     }

//     Ok(collection_and_bid_list)
// }

pub fn update_liquidation_storages(
    storage: &mut dyn Storage,
    liquidation_set: &Vec<BiddedCollateralItem>,
) -> StdResult<Uint128> {
    let mut current_liquidation_value = Uint128::zero();

    for BiddedCollateralItem {
        collection,
        token_id,
        owner,
        liquidator,
        liquidation_price,
        // is_offer, // TODO: add offers handler
        ..
    } in liquidation_set
    {
        // update current_liquidation_value
        current_liquidation_value += liquidation_price;

        // update bids
        let bids: Vec<LiquidationBid> = LIQUIDATION_BIDS
            .load(storage, collection)?
            .into_iter()
            .map(|mut bid| {
                if bid.token_id_list.contains(token_id) {
                    bid.amount -= liquidation_price;
                    bid.token_id_list.retain(|id| id != token_id);
                }

                bid
            })
            .collect();

        LIQUIDATION_BIDS.save(storage, collection, &bids)?;

        // update collaterals
        let mut collaterals = COLLATERALS.load(storage, collection)?;

        let collaterals_by_liquidator = collaterals.iter().find(|x| x.owner == liquidator);
        if collaterals_by_liquidator.is_none() {
            collaterals.push(Collateral {
                owner: liquidator.to_owned(),
                token_id_list: vec![],
            });
        }

        collaterals = collaterals
            .into_iter()
            .map(|mut x| {
                if x.owner == owner {
                    x.token_id_list.retain(|id| id != token_id);
                } else if x.owner == liquidator {
                    x.token_id_list.push(token_id.to_string());
                }

                x
            })
            .filter(|x| !x.token_id_list.is_empty())
            .collect();

        COLLATERALS.save(storage, collection, &collaterals)?;
    }

    Ok(current_liquidation_value)
}

pub fn update_borrowers_storages(
    storage: &mut dyn Storage,
    loan: &Vault,
    borrower_address: &Addr,
    current_liquidation_value: Uint128,
    accumulated_loan: Uint128,
    block_time: u64,
    borrow_apr: Decimal,
) -> StdResult<()> {
    let accumulated_loan = math::calc_accumulated_loan(
        borrow_apr,
        accumulated_loan,
        loan.amount,
        block_time,
        loan.creation_date,
    );

    let (amount, accumulated_loan) = if current_liquidation_value <= loan.amount {
        (loan.amount - current_liquidation_value, accumulated_loan)
    } else {
        let amount_difference = current_liquidation_value - loan.amount;
        (
            Uint128::zero(),
            max(accumulated_loan, amount_difference) - amount_difference,
        )
    };

    // update borrowers
    BORROWERS.save(
        storage,
        borrower_address,
        &Borrower {
            accumulated_loan,
            loan: Vault {
                creation_date: block_time,
                amount,
            },
        },
    )?;

    Ok(())
}

// TODO: should rewards be paid first?
pub fn update_balances(
    storage: &mut dyn Storage,
    block_time: u64,
    borrow_apr: Decimal,
    borrow_fee_rate: Decimal,
    main_currency_pool_addition: Uint128,
    total_liquidation_value: Uint128,
    total_liquidation_bids: Uint128,
) -> StdResult<()> {
    // update contract balances
    BALANCES.update(storage, |mut balances| -> StdResult<Balances> {
        let regular_rewards = math::calc_expected_rewards(
            block_time,
            balances.regular.borrowed.creation_date,
            borrow_apr,
            borrow_fee_rate,
            balances.regular.borrowed.amount,
        );

        let reserve_rewards = math::calc_expected_rewards(
            block_time,
            balances.reserve.borrowed.creation_date,
            borrow_apr,
            borrow_fee_rate,
            balances.reserve.borrowed.amount,
        );

        let rewards = balances.regular.accumulated_rewards + regular_rewards + reserve_rewards;

        if balances.reserve.borrowed.amount.is_zero() {
            if total_liquidation_value <= balances.regular.borrowed.amount {
                balances.regular.borrowed.amount -= total_liquidation_value;
            } else {
                let amount_difference = total_liquidation_value - balances.regular.borrowed.amount;

                balances.regular.borrowed.creation_date = block_time;
                balances.regular.borrowed.amount = Uint128::zero();
                balances.regular.accumulated_rewards =
                    max(rewards, amount_difference) - amount_difference;
            }
        } else {
            if total_liquidation_value <= balances.reserve.borrowed.amount {
                balances.reserve.borrowed.amount -= total_liquidation_value;
            } else if total_liquidation_value
                <= balances.reserve.borrowed.amount + balances.regular.borrowed.amount
            {
                let amount_difference = total_liquidation_value - balances.reserve.borrowed.amount;

                balances.reserve.borrowed.creation_date = block_time;
                balances.reserve.borrowed.amount = Uint128::zero();

                balances.regular.borrowed.amount -= amount_difference;
            } else {
                let amount_difference = total_liquidation_value
                    - balances.reserve.borrowed.amount
                    - balances.regular.borrowed.amount;

                balances.reserve.borrowed.creation_date = block_time;
                balances.reserve.borrowed.amount = Uint128::zero();

                balances.regular.borrowed.creation_date = block_time;
                balances.regular.borrowed.amount = Uint128::zero();
                balances.regular.accumulated_rewards =
                    max(rewards, amount_difference) - amount_difference;
            }
        }

        balances.regular.main_pool += main_currency_pool_addition;
        balances.regular.liquidation_bids -= total_liquidation_bids;

        Ok(balances)
    })?;

    Ok(())
}
