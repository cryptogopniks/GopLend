use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;
use cw_gopniks::assets::Currency;

use goplend_base::{
    converters::str_to_dec,
    error::ContractError,
    lending_platform::{
        msg::InstantiateMsg,
        state::{
            ADDRESS_CONFIG, BALANCES, BID_MIN_RATE, BORROWERS_RESERVE_FRACTION_RATIO, BORROW_APR,
            BORROW_FEE_RATE, COLLATERAL_MIN_VALUE, COMMON_CONFIG, CONTRACT_NAME, DISCOUNT_MAX_RATE,
            DISCOUNT_MIN_RATE, IS_PAUSED, LIQUIDATION_FEE_RATE, PLATFORM_REVENUE, PROPOSAL_COUNTER,
            RATE_CONFIG, UNBONDING_PERIOD,
        },
        types::{AddressConfig, Balances, CommonConfig, RateConfig},
    },
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let sender = &info.sender;
    let block_time = env.block.time.seconds();

    let main_currency = Currency::new(
        &msg.main_currency.token.verify(deps.api)?,
        msg.main_currency.decimals,
    );

    let bgl_currency = msg
        .bgl_currency
        .map(|x| -> StdResult<_> { Ok(Currency::new(&x.token.verify(deps.api)?, x.decimals)) })
        .transpose()?;

    ADDRESS_CONFIG.save(
        deps.storage,
        &AddressConfig {
            admin: sender.to_owned(),
            worker: msg
                .worker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            oracle: msg.oracle.map(|x| deps.api.addr_validate(&x)).transpose()?,
            minter: msg.minter.map(|x| deps.api.addr_validate(&x)).transpose()?,
            scheduler: msg
                .scheduler
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
            market_maker: msg
                .market_maker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()?,
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
            borrowers_reserve_fraction_ratio: msg
                .borrowers_reserve_fraction_ratio
                .unwrap_or(str_to_dec(BORROWERS_RESERVE_FRACTION_RATIO)),
        },
    )?;

    let mut balances = Balances::default();
    balances.regular.borrowed.creation_date = block_time;
    balances.reserve.borrowed.creation_date = block_time;
    BALANCES.save(deps.storage, &balances)?;

    PROPOSAL_COUNTER.save(deps.storage, &1)?;
    PLATFORM_REVENUE.save(deps.storage, &vec![])?;
    IS_PAUSED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
