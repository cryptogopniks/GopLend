use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub lending_platform: Option<String>,
    pub marketplace: Option<String>,
    pub infinity_pool: Option<String>,
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
        marketplace: Option<String>,
        infinity_pool: Option<String>,
    },

    // lending platform
    AcceptBids {
        bidded_collateral_item_list: Vec<crate::lending_platform::types::BiddedCollateralItem>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<crate::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryLiquidationBidsByCollectionAddressList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Vec<crate::lending_platform::types::LiquidationBid>)]
    QueryLiquidationBidsByCollectionAddress { address: String },

    #[returns(Vec<crate::lending_platform::msg::QueryLiquidationBidsByLiquidatorAddressListResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddressList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Vec<crate::lending_platform::msg::QueryLiquidationBidsByCollectionAddressListResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddress { address: String },
}
