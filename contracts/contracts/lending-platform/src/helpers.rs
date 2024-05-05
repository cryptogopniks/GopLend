use std::cmp::max;

use cosmwasm_std::{Addr, Decimal, Deps, StdResult, Storage, Uint128};

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
    let AddressConfig { admin, worker, .. } = ADDRESS_CONFIG.load(deps.storage)?;
    let worker = unwrap_field(worker, "worker");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorker => {
            if !((sender == admin) || (worker.is_ok() && sender == worker?)) {
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
        AuthType::AdminOrWorkerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (worker.is_ok() && sender == worker?) || is_included) {
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

pub fn get_balances_with_updated_rewards(
    storage: &dyn Storage,
    block_time: u64,
) -> StdResult<Balances> {
    let rate_config = RATE_CONFIG.load(storage)?;
    let mut balances = BALANCES.load(storage)?;

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
}

pub fn accumulate_expected_rewards(storage: &mut dyn Storage, block_time: u64) -> StdResult<()> {
    let balances = get_balances_with_updated_rewards(storage, block_time)?;
    BALANCES.save(storage, &balances)?;

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
            .filter(|bid| !bid.amount.is_zero() || !bid.token_id_list.is_empty())
            .collect();

        let bids_sum = bids
            .iter()
            .fold(Uint128::zero(), |acc, cur| acc + cur.amount);
        let tokens_sum = Uint128::new(
            bids.iter()
                .fold(0, |acc, cur| acc + cur.token_id_list.len()) as u128,
        );

        if bids_sum.is_zero() && tokens_sum.is_zero() {
            LIQUIDATION_BIDS.remove(storage, collection);
        } else {
            LIQUIDATION_BIDS.save(storage, collection, &bids)?;
        }

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
    if (amount + accumulated_loan).is_zero() {
        BORROWERS.remove(storage, borrower_address);
    } else {
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
    }

    Ok(())
}

pub fn update_balances(
    storage: &mut dyn Storage,
    block_time: u64,
    borrow_apr: Decimal,
    borrow_fee_rate: Decimal,
    mut main_currency_pool_addition: Uint128,
    total_liquidation_value: Uint128,
) -> StdResult<()> {
    // update contract balances
    BALANCES.update(storage, |mut balances| -> StdResult<Balances> {
        balances.regular.liquidation_bids -= total_liquidation_value;

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

        balances.regular.accumulated_rewards = rewards;
        let accumulated_rewards_diff = std::cmp::min(
            main_currency_pool_addition,
            balances.regular.accumulated_rewards,
        );
        balances.regular.accumulated_rewards -= accumulated_rewards_diff;
        balances.regular.main_pool += accumulated_rewards_diff; // main pool can be increased only by rewards amount
        main_currency_pool_addition -= accumulated_rewards_diff;

        if main_currency_pool_addition <= balances.reserve.withdrawn {
            balances.reserve.withdrawn -= main_currency_pool_addition;

            return Ok(balances);
        }

        balances.reserve.withdrawn = Uint128::zero();

        if main_currency_pool_addition <= balances.reserve.borrowed.amount {
            balances.reserve.borrowed.amount -= main_currency_pool_addition;

            return Ok(balances);
        }

        main_currency_pool_addition -= balances.reserve.borrowed.amount;
        balances.reserve.borrowed.amount = Uint128::zero();

        if balances.regular.borrowed.amount >= main_currency_pool_addition {
            balances.regular.borrowed.amount -= main_currency_pool_addition;
        } else {
            balances.regular.main_pool +=
                main_currency_pool_addition - balances.regular.borrowed.amount;
            balances.regular.borrowed.amount = Uint128::zero();
        }

        Ok(balances)
    })?;

    Ok(())
}

pub mod offchain {
    use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Uint128};

    use goplend_base::{
        error::ContractError,
        lending_platform::{
            msg::{
                QueryBorrowersResponseItem, QueryCollateralsResponseItem,
                QueryLiquidationBidsByCollectionAddressListResponseItem, QueryMsg,
            },
            types::{Collateral, LiquidationBid},
        },
        oracle::{self, types::PriceItem},
    };

    pub fn get_borrowers(
        querier_wrapper: &QuerierWrapper,
        lending_platform_address: &Addr,
        borrowers: &[String],
    ) -> StdResult<Vec<QueryBorrowersResponseItem>> {
        let mut borrower_list = querier_wrapper
            .query_wasm_smart::<Vec<QueryBorrowersResponseItem>>(
                lending_platform_address,
                &QueryMsg::QueryBorrowerList {
                    start_after: None,
                    limit: None,
                },
            )?;
        borrower_list.retain(|x| borrowers.contains(&x.address.to_string()));

        Ok(borrower_list)
    }

    pub fn get_collection_prices(
        querier_wrapper: &QuerierWrapper,
        oracle_address: &Addr,
        borrower_list: &[QueryBorrowersResponseItem],
        collection_and_collateral_list: &[QueryCollateralsResponseItem],
    ) -> StdResult<Vec<PriceItem>> {
        // collect collection addresses over current page borrowers
        let borrower_addresses: Vec<Addr> =
            borrower_list.iter().map(|x| x.address.to_owned()).collect();

        let mut collection_addresses_by_borrowers: Vec<Addr> = collection_and_collateral_list
            .iter()
            .filter(|x| {
                x.collateral
                    .iter()
                    .any(|y| borrower_addresses.contains(&y.owner))
            })
            .map(|x| x.address.to_owned())
            .collect();

        // dedup
        collection_addresses_by_borrowers.sort_unstable();
        collection_addresses_by_borrowers.dedup();

        let collection_addresses = Some(
            collection_addresses_by_borrowers
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        );

        querier_wrapper.query_wasm_smart::<Vec<PriceItem>>(
            oracle_address,
            &oracle::msg::QueryMsg::QueryPrices {
                collection_addresses,
                start_after: None,
                limit: None,
            },
        )
    }

    pub fn get_collaterals_by_borrower(
        collection_and_collateral_list: &[(Addr, Vec<Collateral>)],
        borrower_address: &Addr,
    ) -> StdResult<Vec<(Addr, Collateral)>> {
        // get collaterals owned by borrower
        let collection_and_collateral_list_by_borrower: Vec<(Addr, Collateral)> =
            collection_and_collateral_list
                .iter()
                .cloned()
                .map(
                    |(collection, collaterals)| -> StdResult<(Addr, Collateral)> {
                        // single collateral per collection
                        let collateral = collaterals
                            .into_iter()
                            .find(|collateral| collateral.owner == borrower_address)
                            .ok_or(ContractError::CollateralIsNotFound)?;

                        Ok((collection, collateral))
                    },
                )
                .collect::<StdResult<Vec<(Addr, Collateral)>>>()?;

        Ok(collection_and_collateral_list_by_borrower)
    }

    pub fn get_collateral_value(
        collection_and_collateral_list_by_borrower: &[(Addr, Collateral)],
        prices: &[PriceItem],
        block_time: u64,
        price_update_period: u64,
    ) -> StdResult<Option<Uint128>> {
        let mut is_price_outdated = false;

        let collateral_value = collection_and_collateral_list_by_borrower.iter().fold(
            Ok(Uint128::zero()),
            |acc, (collection_address, collateral)| -> StdResult<Uint128> {
                let oracle::types::PriceItem {
                    price,
                    price_update_date,
                    ..
                } = prices
                    .iter()
                    .find(|x| x.address == collection_address)
                    .ok_or(ContractError::PriceIsNotFound {
                        val: collection_address.to_string(),
                    })?;

                if block_time - price_update_date > price_update_period {
                    is_price_outdated = true;
                }

                Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
            },
        )?;

        Ok(if is_price_outdated {
            None
        } else {
            Some(collateral_value)
        })
    }

    // find bids containing at least 1 borrower's collateral token
    pub fn get_collection_and_bid_list(
        liquidation_bids: &[QueryLiquidationBidsByCollectionAddressListResponseItem],
        collection_and_collateral_list_by_borrower: &[(Addr, Collateral)],
    ) -> StdResult<Vec<(Addr, LiquidationBid)>> {
        let mut collection_and_bid_list: Vec<(Addr, LiquidationBid)> = vec![];

        for (collection_address, collateral) in collection_and_collateral_list_by_borrower {
            let liquidation_bids_by_collection_address = liquidation_bids
                .iter()
                .find(|x| x.collection_address == collection_address);
            if liquidation_bids_by_collection_address.is_none() {
                continue;
            }

            for bid in &liquidation_bids_by_collection_address
                .unwrap()
                .liquidation_bids
            {
                if bid
                    .token_id_list
                    .iter()
                    .any(|id| collateral.token_id_list.contains(id))
                {
                    collection_and_bid_list.push((collection_address.to_owned(), bid.to_owned()));
                }
            }
        }

        Ok(collection_and_bid_list)
    }
}
