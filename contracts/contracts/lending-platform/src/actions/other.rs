use std::cmp::max;

use cosmwasm_std::{
    Addr, Decimal, DepsMut, Env, Order, Response, StdResult, Storage, SubMsgResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};

use cw_storage_plus::Bound;
use semver::Version;

use goplend_base::{
    assets::{Currency, Token},
    converters::u128_to_dec,
    error::ContractError,
    lending_platform::{
        msg::{MigrateMsg, SudoMsg},
        state::{
            BALANCES, BGL_CURRENCY_DECIMALS, BORROWERS, COLLATERALS, CONFIG, CONTRACT_NAME,
            LIQUIDATION_BIDS, PAGINATION_CONFIG,
        },
        types::{
            Balances, Borrower, Collateral, Config, LiquidatedPosition, LiquidationBid, Vault,
        },
    },
    oracle::{self, state::PRICE_UPDATE_PERIOD},
    utils::unwrap_field,
};

use crate::{
    helpers::{check_lockout, increase_platform_revenue},
    math::{
        calc_accumulated_loan, calc_bid_max_multiplier, calc_expected_rewards,
        calc_liquidation_value, calc_ltv, calc_platform_revenue_on_liquidation,
    },
};

pub fn save_cw20_address(
    deps: DepsMut,
    _env: Env,
    result: &SubMsgResult,
) -> Result<Response, ContractError> {
    let res = result
        .to_owned()
        .into_result()
        .map_err(|e| ContractError::CustomError { val: e })?;

    let instantiate_event = unwrap_field(
        res.events.iter().find(|x| x.ty == "instantiate"),
        "instantiate_event",
    )?;

    let cw20_address = &unwrap_field(
        instantiate_event
            .attributes
            .iter()
            .find(|x| x.key == "_contract_address"),
        "cw20_address",
    )?
    .value;

    let bgl_currency = Currency::new(
        &Token::new_cw20(&deps.api.addr_validate(cw20_address)?),
        BGL_CURRENCY_DECIMALS,
    );

    CONFIG.update(deps.storage, |mut x| -> StdResult<Config> {
        x.bgl_currency = Some(bgl_currency);
        Ok(x)
    })?;

    Ok(Response::new().add_attributes([("cw20_address", cw20_address)]))
}

pub fn try_process(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    // skip endblock event
    if let SudoMsg::EndBlock {} = msg {
        return Ok(Response::new().add_attributes(vec![("action", "try_process")]));
    }

    check_lockout(deps.as_ref())?;

    let block_time = env.block.time.seconds();
    let Config {
        main_currency,
        borrow_apr,
        oracle,
        discount_max_rate,
        discount_min_rate,
        bid_min_rate,
        liquidation_fee_rate,
        ..
    } = CONFIG.load(deps.storage)?;
    let oracle = &unwrap_field(oracle, "oracle")?;

    // rotate pagination
    let mut pagination_config = PAGINATION_CONFIG.load(deps.storage)?;
    let pagination_amount = pagination_config.get_amount();

    let address;
    let start_bound = match pagination_config.pagination_index {
        None => None,
        Some(x) => {
            address = x;
            Some(Bound::exclusive(&address))
        }
    };

    let borrowers: Vec<(Addr, Borrower)> = BORROWERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(pagination_amount as usize)
        .flatten()
        .collect();

    let borrower_last = borrowers
        .last()
        .map(|(pagination_index_new, _)| pagination_index_new.to_owned());
    let storage_last = BORROWERS
        .last(deps.storage)?
        .map(|(pagination_index_new, _)| pagination_index_new);

    let pagination_index_new = if borrower_last == storage_last {
        None
    } else {
        borrower_last
    };

    pagination_config.pagination_index = pagination_index_new;
    PAGINATION_CONFIG.save(deps.storage, &pagination_config)?;

    // get all collaterals
    let collection_and_collateral_list: Vec<(Addr, Vec<Collateral>)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    // collect collection addresses over current page borrowers
    let borrower_addresses: Vec<Addr> = borrowers
        .iter()
        .map(|(address, _)| address.to_owned())
        .collect();

    let mut collection_addresses_by_borrowers: Vec<String> = collection_and_collateral_list
        .iter()
        .filter(|(_collection_address, collaterals)| {
            collaterals
                .iter()
                .any(|collateral| borrower_addresses.contains(&collateral.owner))
        })
        .map(|(collection_address, _)| collection_address.to_string())
        .collect();

    collection_addresses_by_borrowers.sort_unstable();
    collection_addresses_by_borrowers.dedup();

    // query collection prices
    let prices: Vec<oracle::msg::QueryPricesResponseItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses_by_borrowers),
        },
    )?;

    // accumulators
    let mut total_liquidation_value = Uint128::zero();
    let mut total_liquidation_bids = Uint128::zero();

    // iterate over borrowers
    for (
        borrower_address,
        Borrower {
            accumulated_loan,
            loan,
        },
    ) in borrowers
    {
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

        let mut is_price_outdated = false;

        let collateral_value = collection_and_collateral_list_by_borrower.iter().fold(
            Ok(Uint128::zero()),
            |acc, (collection_address, collateral)| -> StdResult<Uint128> {
                let oracle::msg::QueryPricesResponseItem {
                    price,
                    price_update_date,
                    ..
                } = prices
                    .iter()
                    .find(|x| x.address == collection_address)
                    .ok_or(ContractError::PriceIsNotFound {
                        val: collection_address.to_string(),
                    })?;

                if block_time - price_update_date > PRICE_UPDATE_PERIOD {
                    is_price_outdated = true;
                }

                // TODO: decimals, other currencies
                Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
            },
        )?;

        // go to next borrower if any price is outdated
        if is_price_outdated {
            continue;
        }

        // check ltv
        let ltv = calc_ltv(
            borrow_apr,
            accumulated_loan,
            loan.amount,
            collateral_value,
            block_time,
            loan.creation_date,
        );

        // if low liquidate worst part and check again in loop
        let ltv_max = calc_bid_max_multiplier(bid_min_rate, discount_max_rate, discount_min_rate);

        // no liquidation, check next borrower
        if ltv <= ltv_max {
            continue;
        }

        // TODO: doesn't allow big fraction liquidation
        // calculate how much tokens and what collection must be liquidated to get proper ltv
        let liquidation_value = calc_liquidation_value(collateral_value, ltv, ltv_max);

        // find bids containing at least 1 borrower's collateral token
        let mut collection_and_bid_list: Vec<(Addr, LiquidationBid)> = vec![];

        for (collection_address, collateral) in &collection_and_collateral_list_by_borrower {
            for bid in LIQUIDATION_BIDS.load(deps.storage, &collection_address)? {
                if bid
                    .token_id_list
                    .iter()
                    .any(|id| collateral.token_id_list.contains(id))
                {
                    collection_and_bid_list.push((collection_address.clone(), bid));
                }
            }
        }

        // TODO: if collection_and_bid_list.is_empty() then accept offers

        // sort bids by discount and creation date
        collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.amount);
        collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.creation_date);
        collection_and_bid_list.sort_unstable_by_key(|(_, bid)| bid.discount);

        // find min list of bids to get proper ltv
        let mut current_liquidation_value = Uint128::zero();
        let mut liquidated_collection_token_and_price_list: Vec<LiquidatedPosition> = vec![];

        for (collection_address, mut bid) in collection_and_bid_list {
            if current_liquidation_value >= liquidation_value {
                break;
            }

            let (
                _,
                Collateral {
                    token_id_list: token_list_in_collateral,
                    ..
                },
            ) = collection_and_collateral_list_by_borrower
                .iter()
                .find(|(collection, _)| collection == collection_address)
                .ok_or(ContractError::CollectionIsNotFound)?;

            // common tokens for collateral and bid
            let mut target_tokens: Vec<String> = bid
                .token_id_list
                .into_iter()
                .filter(|id| token_list_in_collateral.contains(id))
                .rev()
                .collect();

            if target_tokens.is_empty() {
                continue;
            }

            let oracle::msg::QueryPricesResponseItem { price, .. } = prices
                .iter()
                .find(|x| x.address == collection_address)
                .ok_or(ContractError::PriceIsNotFound {
                    val: collection_address.to_string(),
                })?;

            let discounted_price =
                (u128_to_dec(price.amount) * (Decimal::one() - bid.discount)).to_uint_ceil();

            while !target_tokens.is_empty()
                && bid.amount >= discounted_price
                && current_liquidation_value < liquidation_value
            {
                let removed_target_token = target_tokens.pop(); // list reversed before
                if let Some(id) = removed_target_token {
                    liquidated_collection_token_and_price_list.push(LiquidatedPosition {
                        token_id: id,
                        liquidation_price: discounted_price,
                        liquidator: bid.liquidator.clone(),
                    });
                }

                total_liquidation_bids += discounted_price;
                bid.amount -= discounted_price;
                current_liquidation_value += price.amount;
            }

            // TODO: if current_liquidation_value < liquidation_value then accept offers partially

            for LiquidatedPosition {
                token_id,
                liquidation_price,
                liquidator,
            } in &liquidated_collection_token_and_price_list
            {
                // update bids
                let bids: Vec<LiquidationBid> = LIQUIDATION_BIDS
                    .load(deps.storage, &collection_address)?
                    .into_iter()
                    .map(|mut bid| {
                        bid.amount -= liquidation_price;
                        bid.token_id_list = bid
                            .token_id_list
                            .into_iter()
                            .filter(|id| id != token_id)
                            .collect();

                        bid
                    })
                    .collect();

                LIQUIDATION_BIDS.save(deps.storage, &collection_address, &bids)?;

                // update collaterals
                let collaterals: Vec<Collateral> = COLLATERALS
                    .load(deps.storage, &collection_address)?
                    .into_iter()
                    .map(|mut x| {
                        if x.owner == borrower_address {
                            x.token_id_list = x
                                .token_id_list
                                .into_iter()
                                .filter(|id| id != token_id)
                                .collect();

                            x
                        } else if x.owner == liquidator {
                            x.token_id_list.push(token_id.to_string());

                            x
                        } else {
                            x
                        }
                    })
                    .filter(|x| !x.token_id_list.is_empty())
                    .collect();

                COLLATERALS.save(deps.storage, &collection_address, &collaterals)?;
            }
        }

        total_liquidation_value += current_liquidation_value;

        let accumulated_loan = calc_accumulated_loan(
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
            deps.storage,
            &borrower_address,
            &Borrower {
                accumulated_loan,
                loan: Vault {
                    creation_date: block_time,
                    amount,
                },
            },
        )?;
    }

    // collect platform fee
    let platform_revenue =
        calc_platform_revenue_on_liquidation(total_liquidation_value, liquidation_fee_rate);
    let main_currency_pool_addition = total_liquidation_value - platform_revenue;

    increase_platform_revenue(
        deps.storage,
        platform_revenue,
        &main_currency.token,
        &main_currency,
    )?;

    // update contract balances
    BALANCES.update(deps.storage, |mut balances| -> StdResult<Balances> {
        let expected_rewards = calc_expected_rewards(
            block_time,
            balances.total_borrowed.creation_date,
            borrow_apr,
            balances.total_borrowed.amount,
        );
        let rewards = balances.expected_accumulated_rewards + expected_rewards;

        let (amount, rewards) = if total_liquidation_value <= balances.total_borrowed.amount {
            (
                balances.total_borrowed.amount - total_liquidation_value,
                rewards,
            )
        } else {
            let amount_difference = total_liquidation_value - balances.total_borrowed.amount;
            (
                Uint128::zero(),
                max(rewards, amount_difference) - amount_difference,
            )
        };

        balances.total_main_currency_pool += main_currency_pool_addition;
        balances.total_borrowed = Vault {
            amount,
            creation_date: block_time,
        };
        balances.expected_accumulated_rewards = rewards;
        balances.total_liquidation_bids -= total_liquidation_bids;

        Ok(balances)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "try_process")]))
}

pub fn migrate_contract(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    let (version_previous, version_new) = get_versions(deps.storage, msg)?;

    if version_new >= version_previous {
        set_contract_version(deps.storage, CONTRACT_NAME, version_new.to_string())?;
    }

    Ok(Response::new())
}

fn get_versions(
    storage: &dyn Storage,
    msg: MigrateMsg,
) -> Result<(Version, Version), ContractError> {
    let version_previous: Version = get_contract_version(storage)?
        .version
        .parse()
        .map_err(|_| ContractError::ParsingPrevVersion)?;

    let version_new: Version = env!("CARGO_PKG_VERSION")
        .parse()
        .map_err(|_| ContractError::ParsingNewVersion)?;

    if version_new.to_string() != msg.version {
        Err(ContractError::ImproperMsgVersion)?;
    }

    Ok((version_previous, version_new))
}
