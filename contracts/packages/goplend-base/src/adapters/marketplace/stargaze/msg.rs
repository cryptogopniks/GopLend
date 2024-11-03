use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

use stargaze_marketplace_v2::orders::CollectionBid;

use crate::{adapters::dex::stargaze::types::Route, lending_platform::types::BiddedCollateralItem};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub lending_platform_main: Option<String>,
    pub lending_platform_stable: Option<String>,
    pub oracle: Option<String>,
    pub marketplace: Option<String>,
    pub adapter_dex: Option<String>,
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

    #[returns(Vec<(String, Vec<stargaze_marketplace_v2::orders::Bid>)>)]
    QueryTokenOffers {
        collection_address: String,
        token_list: Vec<String>,
        limit: u32,
    },

    #[returns(QueryCollectionOffersResponse)]
    QueryCollectionOffers {
        collection_address: String,
        limit: u32,
    },
}

#[cw_serde]
pub struct QueryCollectionOffersResponse {
    pub collection_bids: Vec<CollectionBid>,
    pub discount_base: Decimal,
    pub price_ratio_stable: Option<Decimal>,
}
