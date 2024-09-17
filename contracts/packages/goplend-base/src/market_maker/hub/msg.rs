use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::assets::TokenUnverified;

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub marketplace: Option<String>,
    pub controller: Option<Vec<String>>,
    pub token: TokenUnverified,
    pub max_outpost_amount: Option<u8>,
    pub outpost_code_id: Option<u64>,
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
        marketplace: Option<String>,
        controller: Option<Vec<String>>,
        token: Option<TokenUnverified>,
        max_outpost_amount: Option<u8>,
        outpost_code_id: Option<u64>,
    },

    AddOutpost {},

    // admin, worker
    Lock {},

    Unlock {},

    // collection owners
    /// add new collection or rewrite existing
    SetCollection {
        collection_address: String,
        owner_list: Vec<String>,
    },

    RemoveCollections {
        collection_address_list: Vec<String>,
    },

    DepositLiquidity {
        collection_address: String,
    },

    /// can remove offers if it's required to get enough liquidity
    WithdrawLiquidity {
        amount: Uint128,
        collection_address: String,
    },

    /// withdraw all bought NFT's
    WithdrawCollateral {
        collection_address: String,
    },

    // controller
    UpdateOffers {
        collection_address: String,
        from_to_price_list: Vec<(Uint128, Uint128)>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::market_maker::types::HubConfig)]
    QueryConfig {},

    #[returns(Vec<Addr>)]
    QueryOutposts {},

    #[returns(Vec<crate::market_maker::types::CollectionOwners<Addr>>)]
    QueryCollectionOwnersList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::market_maker::types::CollectionOwners<Addr>)]
    QueryCollectionOwners { collection_address: String },

    #[returns(Vec<crate::market_maker::types::LiquidityInfo>)]
    QueryLiquidityList {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(crate::market_maker::types::LiquidityInfo)]
    QueryLiquidity { collection_address: String },

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
