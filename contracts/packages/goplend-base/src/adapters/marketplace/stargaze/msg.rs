use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use sg_marketplace::{
    msg::{BidOffset, CollectionBidOffset},
    state::CollectionBid,
};

use crate::{
    adapters::dex::stargaze::types::Route,
    lending_platform::types::{BiddedCollateralItem, LiquidationBid},
};

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

    #[returns(Vec<(Addr, Uint128)>)]
    QueryFloorPrices {
        collection_address_list: Vec<String>,
    },

    #[returns(Vec<(u32, Vec<sg_marketplace::state::Bid>)>)]
    QueryTokenOffers {
        collection_address: String,
        token_list: Vec<u32>,
        limit: u32,
    },

    #[returns(QueryCollectionOffersResponse)]
    QueryCollectionOffers {
        collection_address: String,
        limit: u32,
    },

    #[returns(QueryOffersByCollectionAddressResponse)]
    QueryOffersByCollectionAddress {
        platform_type: PlatformType,
        collection_address: String,
        /// to skip all offers that will be expired during paginated queries
        expiration_delay: u64,
        start_after_collection_offer: Option<CollectionBidOffset>,
        start_after_token_offer: Option<BidOffset>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct QueryOffersByCollectionAddressResponse {
    pub offers: Vec<LiquidationBid>,
    pub last_collection_offer: Option<CollectionBidOffset>,
    pub last_token_offer: Option<BidOffset>,
}

#[cw_serde]
pub struct QueryCollectionOffersResponse {
    pub collection_bids: Vec<CollectionBid>,
    pub discount_base: Decimal,
    pub price_ratio_stable: Decimal,
}
