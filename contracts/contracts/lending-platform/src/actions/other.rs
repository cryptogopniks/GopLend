use cosmwasm_std::{
    Addr, DepsMut, Env, Order, Response, StdResult, Storage, SubMsgResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};

use cw_storage_plus::Bound;
use semver::Version;

use goplend_base::{
    assets::{Currency, Token},
    error::ContractError,
    lending_platform::{
        msg::{MigrateMsg, SudoMsg},
        state::{
            BGL_CURRENCY_DECIMALS, BLOCK_COUNTER, BORROWERS, COLLATERALS, CONFIG, CONTRACT_NAME,
            LIQUIDATION_BIDS, PAGINATION_CONFIG, PAGINATION_INDEX,
        },
        types::{Borrower, Collateral, Config, LiquidationBid, PaginationConfig},
    },
    oracle::{self, state::PRICE_UPDATE_PERIOD},
    utils::{check_funds, unwrap_field},
};

use crate::{
    helpers::check_lockout,
    math::{calc_bid_max_multiplier, calc_liquidation_value, calc_ltv},
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
    let collection_and_collateral_list: Vec<(Addr, Collateral)> = COLLATERALS
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
        .filter(|(_collection_address, collateral)| borrower_addresses.contains(&collateral.owner))
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
                .filter(|(_collection, collateral)| collateral.owner == &borrower_address)
                .collect();

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
                    Err(ContractError::OutdatedPrice {
                        val: collection_address.to_string(),
                    })?;
                }

                // TODO: decimals, other currencies
                Ok(acc? + price.amount * Uint128::new(collateral.token_id_list.len() as u128))
            },
        )?;

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

        // calculate how much tokens and what collection must be liquidates to get proper ltv
        let liquidation_value = calc_liquidation_value(collateral_value, ltv, ltv_max);

        // iterate over each collection to decide what to liquidate first
    }

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
