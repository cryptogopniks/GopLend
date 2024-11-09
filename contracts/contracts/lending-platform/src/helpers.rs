use std::cmp::max;

use cosmwasm_std::{Addr, Decimal, Deps, Order, StdResult, Storage, Uint128};

use goplend_base::{
    assets::{Currency, Funds, Token},
    error::ContractError,
    lending_platform::{
        state::{
            BALANCES, BORROWERS, COLLATERALS, IS_PAUSED, LIQUIDATION_BIDS, PLATFORM_REVENUE,
            RATE_CONFIG,
        },
        types::{
            AddressConfig, Balances, BidType, BiddedCollateralItem, Borrower, Collateral,
            CollectionInfo, CommonConfig, LiquidationBid, Proposal, ProposalStatus, RateConfig,
            TokenItem, Vault,
        },
    },
    oracle::{self, types::PriceItem},
    utils::unwrap_field,
};

use crate::math;

pub fn choose_price(
    main_currency: &Currency<Token>,
    price_item: &PriceItem<Funds<Token>>,
) -> Uint128 {
    if main_currency == &price_item.price.currency {
        price_item.price.amount
    } else {
        price_item.usd_price.amount
    }
}

/// user actions are disabled when the contract is paused
pub fn check_pause_state(storage: &dyn Storage) -> StdResult<()> {
    if IS_PAUSED.load(storage)? {
        Err(ContractError::ContractIsPaused)?;
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
    block_time: u64,
) -> StdResult<(Uint128, Uint128)> {
    // bids + offers
    let mut current_liquidation_value = Uint128::zero();
    // bids
    let mut current_internal_liquidation_value = Uint128::zero();

    for BiddedCollateralItem {
        collection,
        token_item,
        owner,
        liquidator,
        liquidation_price,
        bid_type,
        ..
    } in liquidation_set
    {
        // update collaterals
        let mut collaterals = COLLATERALS.load(storage, collection)?;

        let collaterals_by_liquidator = collaterals.iter().find(|x| x.owner == liquidator);
        if matches!(bid_type, BidType::LiquidationBid) && collaterals_by_liquidator.is_none() {
            collaterals.push(Collateral {
                owner: liquidator.to_owned(),
                token_item_list: vec![],
            });
        }

        collaterals = collaterals
            .into_iter()
            .map(|mut x| {
                if x.owner == owner {
                    x.token_item_list.retain(|item| item.id != token_item.id);
                } else if x.owner == liquidator {
                    x.token_item_list.push(TokenItem {
                        id: token_item.id.clone(),
                        creation_date: block_time,
                    });
                }

                x
            })
            .filter(|x| !x.token_item_list.is_empty())
            .collect();

        COLLATERALS.save(storage, collection, &collaterals)?;

        // update current_liquidation_value
        current_liquidation_value += liquidation_price;

        // update bids if it's required
        if matches!(bid_type, BidType::CollectionOffer) {
            continue;
        }

        let bids: Vec<LiquidationBid> = LIQUIDATION_BIDS
            .load(storage, collection)?
            .into_iter()
            .map(|mut bid| {
                if bid.token_id_list.contains(&token_item.id) {
                    bid.amount -= liquidation_price;
                    bid.token_id_list.retain(|id| id != &token_item.id);
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

        // update current_internal_liquidation_value
        current_internal_liquidation_value += liquidation_price;
    }

    Ok((
        current_liquidation_value,
        current_internal_liquidation_value,
    ))
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
    total_internal_liquidation_value: Uint128,
) -> StdResult<()> {
    // update contract balances
    BALANCES.update(storage, |mut balances| -> StdResult<Balances> {
        balances.regular.liquidation_bids -= total_internal_liquidation_value;

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

pub fn get_collaterals_by_borrower(
    collection_and_collateral_list: &[(Addr, Vec<Collateral>)],
    borrower_address: &Addr,
) -> Vec<(Addr, Collateral)> {
    // get collaterals owned by borrower
    let mut collection_and_collateral_list_by_borrower: Vec<(Addr, Collateral)> = vec![];

    for (collection, collaterals) in collection_and_collateral_list {
        // single collateral per collection
        if let Some(collateral) = collaterals
            .iter()
            .find(|collateral| collateral.owner == borrower_address)
        {
            collection_and_collateral_list_by_borrower
                .push((collection.to_owned(), collateral.to_owned()));
        }
    }

    collection_and_collateral_list_by_borrower
}

pub fn get_collateral_value(
    main_currency: &Currency<Token>,
    collection_and_collateral_list_by_borrower: &[(Addr, Collateral)],
    prices: &[PriceItem<Funds<Token>>],
) -> StdResult<Uint128> {
    let collateral_value = collection_and_collateral_list_by_borrower.iter().fold(
        Ok(Uint128::zero()),
        |acc, (collection_address, collateral)| -> StdResult<Uint128> {
            let price_item = prices
                .iter()
                .find(|x| x.address == collection_address)
                .ok_or(ContractError::PriceIsNotFound {
                    val: collection_address.to_string(),
                })?;

            if price_item.is_outdated {
                Err(ContractError::OutdatedPrice {
                    val: collection_address.to_string(),
                })?;
            }

            let price = choose_price(main_currency, price_item);

            Ok(acc? + price * Uint128::new(collateral.token_item_list.len() as u128))
        },
    )?;

    Ok(collateral_value)
}

#[allow(clippy::too_many_arguments)]
pub fn get_conditional_ltv(
    deps: Deps,
    address_config: &AddressConfig,
    rate_config: &RateConfig,
    common_config: &CommonConfig,
    ltv_max: Decimal,
    borrower_address: &Addr,
    borrower: &Borrower,
    block_time: u64,
    amount_to_deposit: Option<Uint128>,
    amount_to_withdraw: Option<Uint128>,
    amount_to_borrow: Option<Uint128>,
    amount_to_repay: Option<Uint128>,
) -> StdResult<Decimal> {
    let oracle = &unwrap_field(address_config.oracle.clone(), "oracle")?;

    // get collaterals owned by borrower
    let collection_and_collateral_list = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Collateral>)>>>()?;
    let collection_and_collateral_list =
        get_collaterals_by_borrower(&collection_and_collateral_list, borrower_address);

    let collection_addresses: Vec<String> = collection_and_collateral_list
        .iter()
        .map(|(collection, _collateral)| collection.to_string())
        .collect();

    let prices: Vec<oracle::types::PriceItem<Funds<Token>>> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses),
            start_after: None,
            amount: 50, // TODO: replace
        },
    )?;

    let collateral_value = get_collateral_value(
        &common_config.main_currency,
        &collection_and_collateral_list,
        &prices,
    )?;

    Ok(math::calc_conditional_ltv(
        ltv_max,
        rate_config.borrow_apr,
        amount_to_borrow.unwrap_or_default(),
        amount_to_repay.unwrap_or_default(),
        borrower.accumulated_loan,
        borrower.loan.amount,
        amount_to_deposit.unwrap_or_default(),
        amount_to_withdraw.unwrap_or_default(),
        collateral_value,
        block_time,
        borrower.loan.creation_date,
    ))
}

pub mod offchain {
    use cosmwasm_std::{Addr, QuerierWrapper, StdResult};

    use goplend_base::{
        assets::{Funds, Token},
        lending_platform::{
            msg::QueryMsg,
            types::{
                Collateral, LiquidationBid, QueryBorrowersResponseItem,
                QueryCollateralsResponseItem,
                QueryLiquidationBidsByCollectionAddressListResponseItem,
            },
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
                    amount: 50,
                    start_after: None,
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
    ) -> StdResult<Vec<PriceItem<Funds<Token>>>> {
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

        querier_wrapper.query_wasm_smart::<Vec<PriceItem<Funds<Token>>>>(
            oracle_address,
            &oracle::msg::QueryMsg::QueryPrices {
                collection_addresses,
                amount: 50, // TODO: replace
                start_after: None,
            },
        )
    }

    // find bids containing at least 1 borrower's collateral token
    pub fn get_collection_and_bid_list(
        liquidation_bids: &[QueryLiquidationBidsByCollectionAddressListResponseItem],
        collection_and_collateral_list_by_borrower: &[(Addr, Collateral)],
    ) -> StdResult<Vec<(Addr, LiquidationBid)>> {
        let mut collection_and_bid_list: Vec<(Addr, LiquidationBid)> = vec![];

        for QueryLiquidationBidsByCollectionAddressListResponseItem {
            collection_address,
            liquidation_bids,
        } in liquidation_bids
        {
            for bid in liquidation_bids {
                if collection_and_collateral_list_by_borrower.iter().any(
                    |(collection, collateral)| {
                        collection == collection_address
                            && bid
                                .token_id_list
                                .iter()
                                .any(|y| collateral.token_item_list.iter().any(|z| &z.id == y))
                    },
                ) {
                    collection_and_bid_list.push((collection_address.to_owned(), bid.to_owned()));
                }
            }
        }

        Ok(collection_and_bid_list)
    }
}
