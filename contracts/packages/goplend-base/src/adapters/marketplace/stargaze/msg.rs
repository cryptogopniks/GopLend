use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

use crate::{
    adapters::dex::stargaze::types::Route,
    assets::{Currency, TokenUnverified},
    lending_platform::types::BiddedCollateralItem,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub lending_platform: Option<String>,
    pub oracle: Option<String>,
    pub marketplace: Option<String>,
    pub dex: Option<String>,
    pub main_currency: Currency<TokenUnverified>,
    pub price_impact: Option<Decimal>,
    pub price_update_period: Option<u64>,
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
        lending_platform: Option<String>,
        oracle: Option<String>,
        marketplace: Option<String>,
        dex: Option<String>,
        main_currency: Option<Currency<TokenUnverified>>,
        price_impact: Option<Decimal>,
        price_update_period: Option<u64>,
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

    #[returns(Vec<crate::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryOffersByCollectionAddressList {
        is_usd_price_required: bool,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
