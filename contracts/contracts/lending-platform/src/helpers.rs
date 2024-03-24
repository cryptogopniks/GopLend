use cosmwasm_std::{Addr, Api, Deps, Order, StdResult, Storage, Uint128};

use cw_storage_plus::Bound;
use goplend_base::{
    assets::{Currency, Funds, Token},
    error::ContractError,
    lending_platform::{
        state::{
            BALANCES, BORROWERS, COLLATERALS, CONFIG, IS_LOCKED, LIQUIDATION_BIDS,
            PAGINATION_CONFIG, PLATFORM_REVENUE,
        },
        types::{
            Balances, BiddedCollateralItem, Borrower, Collateral, CollectionInfo, Config,
            LiquidationBid, Proposal, ProposalStatus,
        },
    },
    oracle,
    utils::{unwrap_field, AuthType},
};

use crate::math::calc_expected_rewards;

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
    let Config { admin, owner, .. } = CONFIG.load(deps.storage)?;
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
    let config = CONFIG.load(storage)?;

    BALANCES.update(storage, |mut balances| -> StdResult<Balances> {
        let rewards = calc_expected_rewards(
            block_time,
            balances.total_borrowed.creation_date,
            config.borrow_apr,
            balances.total_borrowed.amount,
        );

        balances.accumulated_expected_rewards += rewards;
        balances.total_borrowed.creation_date = block_time;

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

pub fn get_borrowers(
    storage: &mut dyn Storage,
    api: &dyn Api,
    borrowers: Option<Vec<String>>,
) -> StdResult<Vec<(Addr, Borrower)>> {
    let borrowers: Vec<(Addr, Borrower)> = match borrowers {
        Some(x) => x
            .iter()
            .map(|y| {
                let address = api.addr_validate(y)?;
                let borrower = BORROWERS.load(storage, &address)?;

                Ok((address, borrower))
            })
            .collect::<StdResult<Vec<(Addr, Borrower)>>>()?,
        // TODO: remove this branch
        None => {
            // rotate pagination
            let mut pagination_config = PAGINATION_CONFIG.load(storage)?;
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
                .range(storage, start_bound, None, Order::Ascending)
                .take(pagination_amount as usize)
                .flatten()
                .collect();

            let borrower_last = borrowers
                .last()
                .map(|(pagination_index_new, _)| pagination_index_new.to_owned());
            let storage_last = BORROWERS
                .last(storage)?
                .map(|(pagination_index_new, _)| pagination_index_new);

            let pagination_index_new = if borrower_last == storage_last {
                None
            } else {
                borrower_last
            };

            pagination_config.pagination_index = pagination_index_new;
            PAGINATION_CONFIG.save(storage, &pagination_config)?;

            borrowers
        }
    };

    Ok(borrowers)
}

pub fn get_collection_prices(
    deps: Deps,
    borrowers: &Vec<(Addr, Borrower)>,
    collection_and_collateral_list: &Vec<(Addr, Vec<Collateral>)>,
    oracle: &Addr,
) -> StdResult<Vec<oracle::types::PriceItem>> {
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
    let prices: Vec<oracle::types::PriceItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: Some(collection_addresses_by_borrowers),
        },
    )?;

    Ok(prices)
}

pub fn update_liquidation_storages(
    storage: &mut dyn Storage,
    liquidation_set: &Vec<BiddedCollateralItem>,
) -> StdResult<()> {
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
        // update bids
        let bids: Vec<LiquidationBid> = LIQUIDATION_BIDS
            .load(storage, &collection)?
            .into_iter()
            .map(|mut bid| {
                if bid.token_id_list.contains(token_id) {
                    bid.amount -= liquidation_price;
                    bid.token_id_list = bid
                        .token_id_list
                        .into_iter()
                        .filter(|id| id != token_id)
                        .collect();
                }

                bid
            })
            .collect();

        LIQUIDATION_BIDS.save(storage, &collection, &bids)?;

        // update collaterals
        let mut collaterals = COLLATERALS.load(storage, &collection)?;

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
                    x.token_id_list = x
                        .token_id_list
                        .into_iter()
                        .filter(|id| id != token_id)
                        .collect();
                } else if x.owner == liquidator {
                    x.token_id_list.push(token_id.to_string());
                }

                x
            })
            .filter(|x| !x.token_id_list.is_empty())
            .collect();

        COLLATERALS.save(storage, &collection, &collaterals)?;
    }

    Ok(())
}
