use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::{assets::TokenUnverified, lending_platform::types::BiddedCollateralItem};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub controller: Option<Vec<String>>,
    pub lending_platform: Option<String>,
    pub oracle: Option<String>,
    pub token: TokenUnverified,
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
        controller: Option<Vec<String>>,
        lending_platform: Option<String>,
        oracle: Option<String>,
        token: Option<TokenUnverified>,
    },

    // admin, worker
    Pause {},

    Unpause {},

    // collection owners
    /// add new collection or rewrite existing
    SetCollection {
        collection: String,
        owner_list: Vec<String>,
    },

    RemoveCollections {
        collection_list: Vec<String>,
    },

    DepositLiquidity {
        collection: String,
    },

    /// can remove offers if it's required to get enough liquidity
    WithdrawLiquidity {
        amount: Uint128,
        collection: String,
    },

    /// withdraw all bought NFT's
    WithdrawCollateral {
        collection: String,
    },

    // controller
    UpdateOffers {
        collection: String,
        from_to_price_list: Vec<(Uint128, Uint128)>,
    },

    // lending platform
    AcceptOffers {
        bidded_collateral_item_list: Vec<BiddedCollateralItem>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    QueryConfig {},

    #[returns(Vec<super::types::OffersResponse>)]
    QueryOfferPricesList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::OffersResponse)]
    QueryOfferPrices { collection: String },

    #[returns(Vec<super::types::CollectionOwners<Addr>>)]
    QueryCollectionOwnersList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::CollectionOwners<Addr>)]
    QueryCollectionOwners { collection: String },

    #[returns(Vec<super::types::LiquidityInfo>)]
    QueryLiquidityList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(super::types::LiquidityInfo)]
    QueryLiquidity { collection: String },

    #[returns(Vec<crate::lending_platform::types::CollectionInfo<Addr>>)]
    QueryCollateralList {
        amount: u32,
        start_after: Option<String>,
    },

    #[returns(crate::lending_platform::types::CollectionInfo<Addr>)]
    QueryCollateral { collection: String },
}
