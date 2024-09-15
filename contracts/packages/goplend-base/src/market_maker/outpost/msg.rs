use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

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
    },

    /// withdraw all bought NFT's
    WithdrawCollateral {
        collection_address: String,
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

    #[returns(crate::lending_platform::types::CollectionInfo<Addr>)]
    QueryCollateral { collection_address: String },

    #[returns(Vec<crate::market_maker::types::OffersResponse>)]
    QueryOffersList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::market_maker::types::OffersResponse)]
    QueryOffers { collection_address: String },
}
