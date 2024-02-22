use cosmwasm_std::{Addr, DepsMut, Env, Order, Response, StdResult, Storage, SubMsgResult};
use cw2::{get_contract_version, set_contract_version};

use cw_storage_plus::Bound;
use semver::Version;

use goplend_base::{
    assets::{Currency, Token},
    error::ContractError,
    lending_platform::{
        msg::{MigrateMsg, SudoMsg},
        state::{
            BGL_CURRENCY_DECIMALS, BLOCK_COUNTER, COLLATERALS, CONFIG, CONTRACT_NAME,
            LIQUIDATION_BIDS, PAGINATION_CONFIG, PAGINATION_INDEX,
        },
        types::{Collateral, Config, LiquidationBid, PaginationConfig},
    },
    oracle,
    utils::{check_funds, unwrap_field},
};

use crate::helpers::check_lockout;

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

    // get collaterals owned by borrower
    let collection_and_collateral_list: Vec<(Addr, Collateral)> = COLLATERALS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    // query collection prices
    let prices: Vec<oracle::msg::QueryPricesResponseItem> = deps.querier.query_wasm_smart(
        oracle,
        &oracle::msg::QueryMsg::QueryPrices {
            collection_addresses: None,
        },
    )?;

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

    let liquidation_bids: Vec<(Addr, Vec<LiquidationBid>)> = LIQUIDATION_BIDS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(pagination_amount as usize)
        .flatten()
        .collect();

    let bids_last = liquidation_bids
        .last()
        .map(|(pagination_index_new, _)| pagination_index_new.to_owned());
    let storage_last = LIQUIDATION_BIDS
        .last(deps.storage)?
        .map(|(pagination_index_new, _)| pagination_index_new);

    let pagination_index_new = if bids_last == storage_last {
        None
    } else {
        bids_last
    };

    pagination_config.pagination_index = pagination_index_new;
    PAGINATION_CONFIG.save(deps.storage, &pagination_config)?;

    // iterate over bids
    for (collection_address, bids) in liquidation_bids {
        // calc ltv

        // aggregate bids by token id
        for bid in bids {}
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
