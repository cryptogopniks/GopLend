use std::cmp::min;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Uint128};

use goplend_base::{
    assets::{Currency, Funds},
    error::ContractError,
    oracle::{
        state::{AVERAGE_OFFER_PRICE_MULTIPLIER, CONFIG, PRICES_MAX_AMOUNT, PRICE_AND_DATE_LIST},
        types::{Config, RawPriceItem},
    },
    utils::{check_funds, unwrap_field, Attrs, FundsType},
};

pub fn try_update_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: Vec<RawPriceItem>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let scheduler = unwrap_field(CONFIG.load(deps.storage)?.scheduler, "scheduler")?;

    if sender_address != scheduler {
        Err(ContractError::Unauthorized)?;
    }

    if data.is_empty() {
        Err(ContractError::EmptyRawPriceItemVector)?;
    }

    for RawPriceItem {
        collection_address,
        currency,
        raw_prices,
        trading_volume_per_day,
    } in data
    {
        if raw_prices.is_empty() {
            Err(ContractError::EmptyPriceVector)?;
        }

        if raw_prices.len() > PRICES_MAX_AMOUNT {
            Err(ContractError::ExceededPricesMaxAmount)?;
        }

        if trading_volume_per_day.is_zero() {
            Err(ContractError::ZeroTradingVolume)?;
        }

        let collection_address = deps.api.addr_validate(&collection_address)?;

        if PRICE_AND_DATE_LIST.has(deps.storage, &collection_address)
            && PRICE_AND_DATE_LIST
                .load(deps.storage, &collection_address)?
                .1
                == block_time
        {
            Err(ContractError::CollectionDuplication)?;
        }

        let currency = Currency::new(&currency.token.verify(deps.api)?, currency.decimals);
        let mut average_offer_price = Uint128::zero();

        for raw_price in raw_prices {
            if raw_price.is_zero() {
                Err(ContractError::ZerosInPrices)?;
            }

            average_offer_price += raw_price;
        }

        average_offer_price /= Uint128::from(PRICES_MAX_AMOUNT as u128);

        let liquidity_divider = Uint128::new(AVERAGE_OFFER_PRICE_MULTIPLIER) * average_offer_price;
        let trading_volume_per_day = min(trading_volume_per_day, liquidity_divider);
        let collateral_price = average_offer_price * trading_volume_per_day / liquidity_divider;

        PRICE_AND_DATE_LIST.save(
            deps.storage,
            &collection_address,
            &(Funds::new(collateral_price, &currency), block_time),
        )?;
    }

    Ok(Response::new().add_attributes(vec![("action", "try_update_prices")]))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    scheduler: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let mut attrs = Attrs::init("try_update_config");

    CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
        if sender_address != config.admin {
            Err(ContractError::Unauthorized)?;
        }

        if let Some(x) = scheduler {
            config.scheduler = Some(deps.api.addr_validate(&x)?);
            attrs.push(("scheduler".to_string(), x));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
}
