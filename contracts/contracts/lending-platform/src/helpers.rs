use cosmwasm_std::{Addr, Deps, StdResult};

use goplend_base::{
    assets::Token,
    error::ContractError,
    lending_platform::{
        state::{CONFIG, IS_LOCKED},
        types::{CollectionInfo, Config, Proposal, ProposalStatus},
    },
    utils::{unwrap_field, AuthType},
};

pub fn check_lockout(deps: Deps) -> StdResult<()> {
    if IS_LOCKED.load(deps.storage)? {
        Err(ContractError::ContractIsLocked)?;
    }

    Ok(())
}

pub fn verify_proposal_status(
    proposal: &Proposal<Addr, Token>,
    expected: ProposalStatus,
) -> StdResult<()> {
    let proposal_status = unwrap_field(proposal.proposal_status.as_ref(), "proposal_status")?;

    if proposal_status != &expected {
        Err(ContractError::WrongProposalStatus)?;
    }

    Ok(())
}

pub fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let Config { admin, owner, .. } = CONFIG.load(deps.storage)?;
    let owner = unwrap_field(owner, "owner");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrOwner => {
            if !((sender == admin) || (owner.is_ok() && sender == owner?)) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::Specified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !is_included {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrOwnerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (owner.is_ok() && sender == owner?) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
    };

    Ok(())
}

pub fn check_collections_holder(
    deps: Deps,
    holder: &Addr,
    collections: &Vec<CollectionInfo<String>>,
) -> StdResult<()> {
    const MAX_LIMIT: u32 = 100;
    const ITER_LIMIT: u32 = 50;

    for CollectionInfo {
        collection_address,
        token_id_list,
    } in collections
    {
        let mut token_list: Vec<String> = vec![];
        let mut token_amount_sum: u32 = 0;
        let mut i: u32 = 0;
        let mut last_token: Option<String> = None;

        while (i == 0 || token_amount_sum == MAX_LIMIT) && i < ITER_LIMIT {
            i += 1;

            let query_tokens_msg = cw721::Cw721QueryMsg::Tokens {
                owner: holder.to_string(),
                start_after: last_token,
                limit: Some(MAX_LIMIT),
            };

            let cw721::TokensResponse { tokens } = deps
                .querier
                .query_wasm_smart(collection_address, &query_tokens_msg)?;

            for token in tokens.clone() {
                token_list.push(token);
            }

            token_amount_sum = tokens.len() as u32;
            last_token = tokens.last().cloned();
        }

        let are_tokens_owned = token_id_list.iter().all(|x| token_list.contains(x));

        if !are_tokens_owned {
            Err(ContractError::NftIsNotFound)?;
        }
    }

    Ok(())
}
