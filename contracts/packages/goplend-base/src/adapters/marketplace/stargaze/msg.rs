use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::{adapters::dex::stargaze::types::Route, lending_platform::types::BiddedCollateralItem};

use super::types::{MarketplaceVersion, PlatformType};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub lending_platform_main: Option<String>,
    pub lending_platform_stable: Option<String>,
    pub oracle: Option<String>,
    pub marketplace: Option<String>,
    pub adapter_dex: Option<String>,
    pub marketplace_version: Option<MarketplaceVersion>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // any
    AcceptAdminRole {},

    // admin
    UpdateConfig {
        admin: Option<String>,
        worker: Option<String>,
        lending_platform_main: Option<String>,
        lending_platform_stable: Option<String>,
        oracle: Option<String>,
        marketplace: Option<String>,
        adapter_dex: Option<String>,
        marketplace_version: Option<MarketplaceVersion>,
    },

    // lending platform
    AcceptOffers {
        bidded_collateral_item_list: Vec<BiddedCollateralItem>,
        route: Route,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<crate::lending_platform::types::LiquidationBid>)]
    QueryOffersByCollectionAddress {
        platform_type: PlatformType,
        collection_address: String,
        /// to skip all offers that will be expired during paginated queries
        expiration_delay: u64,
        start_after_collection_offer: Option<String>,
        start_after_token_offer: Option<String>,
        limit: Option<u32>,
    },
}
