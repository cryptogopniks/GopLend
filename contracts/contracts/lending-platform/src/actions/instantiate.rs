use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;

use goplend_base::{
    assets::{Currency, Token},
    converters::str_to_dec,
    error::ContractError,
    lending_platform::{
        msg::InstantiateMsg,
        state::{
            ADDRESS_CONFIG, BALANCES, BID_MIN_RATE, BORROW_APR, BORROW_FEE_RATE,
            COLLATERAL_MIN_VALUE, COMMON_CONFIG, CONTRACT_NAME, DISCOUNT_MAX_RATE,
            DISCOUNT_MIN_RATE, IS_LOCKED, IS_MARKETPLACE_INTERFACE_ENABLED, LIQUIDATION_FEE_RATE,
            MIN_AVAILABLE_LIQUIDITY_MULTIPLIER, PLATFORM_REVENUE, PRICE_UPDATE_PERIOD,
            PROPOSAL_COUNTER, RATE_CONFIG, UNBONDING_PERIOD,
        },
        types::{AddressConfig, Balances, CommonConfig, RateConfig},
    },
    utils::{validate_attr, Attrs},
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let block_time = env.block.time.seconds();
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

    ADDRESS_CONFIG.save(
        deps.storage,
        &AddressConfig {
            admin: info.sender,
            owner: validate_attr(&mut attrs, api, "owner", &msg.owner)?,
            oracle: validate_attr(&mut attrs, api, "oracle", &msg.oracle)?,
            minter: validate_attr(&mut attrs, api, "minter", &msg.minter)?,
            scheduler: validate_attr(&mut attrs, api, "scheduler", &msg.scheduler)?,
            marketplace: validate_attr(&mut attrs, api, "marketplace", &msg.marketplace)?,
        },
    )?;

    RATE_CONFIG.save(
        deps.storage,
        &RateConfig {
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

    COMMON_CONFIG.save(
        deps.storage,
        &CommonConfig {
            main_currency,
            bgl_currency,
            unbonding_period: msg.unbonding_period.unwrap_or(UNBONDING_PERIOD),
            collateral_min_value: msg
                .collateral_min_value
                .unwrap_or(Uint128::new(COLLATERAL_MIN_VALUE)),
            min_available_liquidity_multiplier: msg
                .min_available_liquidity_multiplier
                .unwrap_or(str_to_dec(MIN_AVAILABLE_LIQUIDITY_MULTIPLIER)),
            price_update_period: msg.price_update_period.unwrap_or(PRICE_UPDATE_PERIOD),
            is_marketplace_interface_enabled: msg
                .is_marketplace_interface_enabled
                .unwrap_or(IS_MARKETPLACE_INTERFACE_ENABLED),
        },
    )?;

    let mut balances = Balances::default();
    balances.regular.borrowed.creation_date = block_time;
    balances.reserve.borrowed.creation_date = block_time;
    BALANCES.save(deps.storage, &balances)?;

    PROPOSAL_COUNTER.save(deps.storage, &1)?;
    PLATFORM_REVENUE.save(deps.storage, &vec![])?;
    IS_LOCKED.save(deps.storage, &false)?;

    Ok(Response::new().add_attributes(attrs))
}
