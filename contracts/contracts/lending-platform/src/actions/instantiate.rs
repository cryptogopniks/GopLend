use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use goplend_base::{
    assets::{Currency, Token},
    converters::str_to_dec,
    error::ContractError,
    lending_platform::{
        msg::InstantiateMsg,
        state::{
            BALANCES, BID_MIN_RATE, BORROW_APR, BORROW_FEE_RATE, CONFIG, CONTRACT_NAME,
            CW20_CODE_ID, DISCOUNT_MAX_RATE, DISCOUNT_MIN_RATE, IS_LOCKED, LIQUIDATION_FEE_RATE,
            UNBONDING_PERIOD,
        },
        types::{Balances, Config},
    },
    utils::{validate_attr, Attrs},
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut attrs = Attrs::init("try_instantiate");
    let api = deps.api;

    let main_currency = Currency::new(
        &msg.main_currency.token.verify(deps.api)?,
        msg.main_currency.decimals,
    );

    let bgl_currency = msg
        .bgl_currency
        .map(|x| -> StdResult<Currency<Token>> {
            Ok(Currency::new(&x.token.verify(deps.api)?, x.decimals))
        })
        .transpose()?;

    CONFIG.save(
        deps.storage,
        &Config {
            admin: info.sender,
            owner: validate_attr(&mut attrs, api, "owner", &msg.owner)?,
            oracle: validate_attr(&mut attrs, api, "oracle", &msg.oracle)?,
            main_currency,
            bgl_currency,
            cw20_base_code_id: msg.cw20_base_code_id.unwrap_or(CW20_CODE_ID),
            unbonding_period: msg.unbonding_period.unwrap_or(UNBONDING_PERIOD),
            borrow_apr: msg.borrow_apr.unwrap_or(str_to_dec(BORROW_APR)),
            borrow_fee_rate: msg.borrow_fee_rate.unwrap_or(str_to_dec(BORROW_FEE_RATE)),
            liquidation_fee_rate: msg
                .liquidation_fee_rate
                .unwrap_or(str_to_dec(LIQUIDATION_FEE_RATE)),
            discount_min_rate: msg
                .discount_min_rate
                .unwrap_or(str_to_dec(DISCOUNT_MIN_RATE)),
            discount_max_rate: msg
                .discount_max_rate
                .unwrap_or(str_to_dec(DISCOUNT_MAX_RATE)),
            bid_min_rate: msg.bid_min_rate.unwrap_or(str_to_dec(BID_MIN_RATE)),
        },
    )?;

    BALANCES.save(deps.storage, &Balances::default())?;

    IS_LOCKED.save(deps.storage, &false)?;

    Ok(Response::new().add_attributes(attrs))
}
