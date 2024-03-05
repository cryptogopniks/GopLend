use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use goplend_base::{
    assets::{Currency, Funds},
    error::ContractError,
    oracle::{
        state::{CONFIG, PRICE_AND_DATE_LIST},
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
        price,
    } in data
    {
        if price.amount.is_zero() {
            Err(ContractError::ZerosInPrices)?;
        }

        let collection_address = deps.api.addr_validate(&collection_address)?;
        let price_token = price.currency.token.verify(deps.api)?;
        let currency = Currency::new(&price_token, price.currency.decimals);
        let price = Funds::new(price.amount, &currency);

        if PRICE_AND_DATE_LIST.has(deps.storage, &collection_address)
            && PRICE_AND_DATE_LIST
                .load(deps.storage, &collection_address)?
                .1
                == block_time
        {
            Err(ContractError::CollectionDuplication)?;
        }

        PRICE_AND_DATE_LIST.save(deps.storage, &collection_address, &(price, block_time))?;
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
