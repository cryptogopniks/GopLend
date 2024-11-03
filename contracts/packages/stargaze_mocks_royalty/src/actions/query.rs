use cosmwasm_std::{Addr, Deps, Env, StdResult};

use goplend_base::converters::str_to_dec;
use sg_index_query::QueryOptions;
use stargaze_royalty_registry::state::{RoyaltyEntry, RoyaltyProtocol};

pub fn query_royalty_protocol_by_collection(
    _deps: Deps,
    _env: Env,
    collection: String,
    _query_options: Option<QueryOptions<String>>,
) -> StdResult<Vec<RoyaltyProtocol>> {
    Ok(vec![RoyaltyProtocol {
        collection: Addr::unchecked(collection),
        protocol: Addr::unchecked("protocol"),
        royalty_entry: RoyaltyEntry {
            share: str_to_dec("0.02"),
            updated: None,
            recipient: Addr::unchecked("recipient"),
        },
    }])
}
