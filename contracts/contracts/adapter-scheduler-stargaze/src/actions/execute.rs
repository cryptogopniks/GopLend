use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use goplend_base::{
    adapters::scheduler::stargaze::{
        state::{CONFIG, LOG},
        types::{ClockSource, Config},
    },
    error::ContractError,
    utils::{check_funds, unwrap_field, Attrs, FundsType},
};

// offchain handler
pub fn try_push(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrowers: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let block_time = env.block.time.seconds();
    let sender = &info.sender;
    let Config {
        lending_platform,
        offchain_clock,
        clock_source,
        min_clock_period,
        ..
    } = CONFIG.load(deps.storage)?;

    if let ClockSource::Onchain = clock_source {
        Err(ContractError::Unauthorized)?;
    }

    if let ClockSource::Offchain = clock_source {
        if !unwrap_field(offchain_clock, "offchain_clock")?.contains(sender) {
            Err(ContractError::Unauthorized)?;
        }
    }

    let (time, cnt) = unwrap_field(LOG.back(deps.storage)?, "log")?;
    let cnt = if cnt == u32::MAX { 0 } else { cnt + 1 };

    if block_time < time + min_clock_period {
        Err(ContractError::EarlyExecution)?;
    }

    if LOG.len(deps.storage)? == u32::MAX - 1 {
        LOG.pop_front(deps.storage)?;
    }
    LOG.push_back(deps.storage, &(block_time, cnt))?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: unwrap_field(lending_platform, "lending_platform")?.to_string(),
        msg: to_json_binary(&goplend_base::lending_platform::msg::ExecuteMsg::Process {
            borrowers,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "try_push")]))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    lending_platform: Option<String>,
    offchain_clock: Option<Vec<String>>,
    clock_source: Option<ClockSource>,
    min_clock_period: Option<u64>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let mut attrs = Attrs::init("try_update_config");

    CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
        if sender_address != config.admin {
            Err(ContractError::Unauthorized)?;
        }

        if let Some(x) = lending_platform {
            config.lending_platform = Some(deps.api.addr_validate(&x)?);
            attrs.push(("lending_platform".to_string(), x));
        }

        if let Some(x) = offchain_clock {
            config.offchain_clock = Some(
                x.into_iter()
                    .map(|y| deps.api.addr_validate(&y))
                    .collect::<StdResult<Vec<Addr>>>()?,
            );
            attrs.push((
                "offchain_clock".to_string(),
                format!("{:#?}", config.offchain_clock),
            ));
        }

        if let Some(x) = clock_source {
            config.clock_source = x.clone();
            attrs.push(("clock_source".to_string(), format!("{:#?}", x)));
        }

        if let Some(x) = min_clock_period {
            config.min_clock_period = x.clone();
            attrs.push(("min_clock_period".to_string(), x.to_string()));
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(attrs))
}
