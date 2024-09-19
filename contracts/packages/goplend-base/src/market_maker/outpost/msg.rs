use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // hub
    SetOffer {
        collection_address: String,
    },

    RemoveOffer {
        collection_address: String,
        recipient: Option<String>,
    },

    /// withdraw all bought NFT's
    WithdrawCollateral {
        collection_address: String,
        recipient: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<crate::lending_platform::types::CollectionInfo<Addr>>)]
    QueryCollateralList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::market_maker::types::CollateralResponse)]
    QueryCollateral { collection_address: String },

    #[returns(Vec<crate::market_maker::types::OffersResponse>)]
    QueryOfferList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Uint128)]
    QueryOffer { collection_address: String },
}
