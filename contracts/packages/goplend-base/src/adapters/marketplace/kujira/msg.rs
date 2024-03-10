use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub lending_platform: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // admin
    UpdateConfig {
        lending_platform: Option<String>,
    },

    // lending platform
    AcceptBids {
        liquidation_bids: Vec<crate::lending_platform::types::LiquidationBid>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<crate::lending_platform::msg::QueryLiquidationBidsResponseItem>)]
    QueryLiquidationBidsByCollectionAddresses { addresses: Option<Vec<String>> },

    #[returns(Vec<crate::lending_platform::msg::QueryLiquidationBidsResponseItem>)]
    QueryLiquidationBidsByLiquidatorAddresses { addresses: Option<Vec<String>> },
}
