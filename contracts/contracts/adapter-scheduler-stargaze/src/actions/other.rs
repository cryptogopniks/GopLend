use cosmwasm_std::{to_json_binary, CosmosMsg, DepsMut, Env, Response, Storage, WasmMsg};
use cw2::{get_contract_version, set_contract_version};

use semver::Version;

use goplend_base::{
    adapters::scheduler::stargaze::{
        msg::{MigrateMsg, SudoMsg},
        state::{CONFIG, CONTRACT_NAME, LOG},
        types::{ClockSource, Config},
    },
    error::ContractError,
    utils::unwrap_field,
};

pub fn try_push(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    // skip endblock event
    if let SudoMsg::EndBlock {} = msg {
        Err(ContractError::EarlyExecution)?;
    }

    let block_time = env.block.time.seconds();
    let Config {
        lending_platform,
        clock_source,
        min_clock_period,
        ..
    } = CONFIG.load(deps.storage)?;

    if let ClockSource::Offchain = clock_source {
        Err(ContractError::Unauthorized)?;
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

    // let msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: unwrap_field(lending_platform, "lending_platform")?.to_string(),
    //     msg: to_json_binary(
    //         &goplend_base::lending_platform::msg::ExecuteMsg::Liquidate { targets },
    //     )?,
    //     funds: vec![],
    // });

    Ok(Response::new()
        //       .add_message(msg)
        .add_attributes(vec![("action", "try_push")]))
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
