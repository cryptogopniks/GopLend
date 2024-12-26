use cosmwasm_std::{Addr, DepsMut, Env, Response, Storage};
use cw2::{get_contract_version, set_contract_version};

use semver::Version;

use goplend_base::{
    error::ContractError,
    lending_platform::types::CollectionInfo,
    market_maker::{
        msg::MigrateMsg,
        state::{COLLATERALS, CONTRACT_NAME},
    },
};

pub fn migrate_contract(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    let (version_previous, version_new) = get_versions(deps.storage, msg)?;

    if version_new >= version_previous {
        set_contract_version(deps.storage, CONTRACT_NAME, version_new.to_string())?;

        COLLATERALS.clear(deps.storage);
        COLLATERALS.save(
            deps.storage,
            &Addr::unchecked("neutron1hvp3q00ypzrurd46h7c7c3hu86tx9uf8qt2q28"),
            &vec![CollectionInfo {
                collection_address: Addr::unchecked(
                    "neutron150u7vt6m068z5ezr776z6rxgwmn4wkhlm57nkl4dqvefpygvx4fs0dedzd",
                ),
                token_id_list: vec!["937".to_string()],
            }],
        )?;
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
