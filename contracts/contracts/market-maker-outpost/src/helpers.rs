use cosmwasm_std::{Addr, Deps, StdResult};

use goplend_base::market_maker::{
    outpost::state::CONFIG,
    types::{HubConfig, OutpostConfig},
};

/// returns (hub_config, hub_address)
pub fn get_config(deps: Deps) -> StdResult<(HubConfig, Addr)> {
    let OutpostConfig { hub } = CONFIG.load(deps.storage)?;
    let config = deps.querier.query_wasm_smart::<HubConfig>(
        &hub,
        &goplend_base::market_maker::hub::msg::QueryMsg::QueryConfig {},
    )?;

    Ok((config, hub))
}
