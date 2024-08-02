use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use cw20::Logo;
use cw20_base::msg::InstantiateMarketingInfo;

use crate::minter::types::Metadata;

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub whitelist: Option<Vec<String>>,
    pub cw20_code_id: Option<u64>,
    pub permissionless_token_creation: Option<bool>,
    pub permissionless_token_registration: Option<bool>,
    pub max_tokens_per_owner: Option<u16>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // any specified ------------------------------------------------------------------------------
    AcceptAdminRole {},

    AcceptTokenOwnerRole {},

    // minter admin ------------------------------------------------------------------------------
    /// disable user actions
    Pause {},

    /// enable user actions
    Unpause {},

    UpdateConfig {
        admin: Option<String>,
        whitelist: Option<Vec<String>>,
        cw20_code_id: Option<u64>,
        permissionless_token_creation: Option<bool>,
        permissionless_token_registration: Option<bool>,
        max_tokens_per_owner: Option<u16>,
    },

    // minter whitelist or any user if permissionless_token_creation is true ---------------------
    CreateNative {
        owner: Option<String>,
        whitelist: Option<Vec<String>>,
        permissionless_burning: Option<bool>,
        subdenom: String,
        decimals: Option<u8>,
    },

    CreateCw20 {
        owner: Option<String>,
        whitelist: Option<Vec<String>>,
        permissionless_burning: Option<bool>,
        cw20_code_id: Option<u64>,
        name: String,
        symbol: String,
        decimals: Option<u8>,
        marketing: Option<InstantiateMarketingInfo>,
    },

    // minter whitelist or any token creator if permissionless_token_registration is true ---------
    RegisterNative {
        denom: String,
        owner: Option<String>,
        whitelist: Option<Vec<String>>,
        permissionless_burning: Option<bool>,
        decimals: Option<u8>,
    },

    RegisterCw20 {
        address: String,
        owner: Option<String>,
        whitelist: Option<Vec<String>>,
        permissionless_burning: Option<bool>,
        cw20_code_id: Option<u64>,
        decimals: Option<u8>,
    },

    // token owner ------------------------------------------------------------------------------
    UpdateCurrencyInfo {
        denom_or_address: String,
        owner: Option<String>,
        whitelist: Option<Vec<String>>,
        permissionless_burning: Option<bool>,
    },

    UpdateMetadataNative {
        denom: String,
        metadata: Metadata,
    },

    UpdateMetadataCw20 {
        address: String,
        project: Option<String>,
        description: Option<String>,
        logo: Option<Logo>,
    },

    ExcludeNative {
        denom: String,
    },

    ExcludeCw20 {
        address: String,
    },

    // token whitelist -------------------------------------------------------------------------
    Mint {
        denom_or_address: String,
        amount: Uint128,
        recipient: Option<String>,
    },

    // token whitelist or any holders (if permissionless burning is enabled) -------------------
    Burn {},

    Receive(cw20::Cw20ReceiveMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::minter::types::Config)]
    Config {},

    #[returns(crate::minter::types::CurrencyInfo)]
    CurrencyInfo { denom_or_address: String },

    #[returns(Vec<crate::minter::types::CurrencyInfo>)]
    CurrencyInfoList {
        amount: u32,
        start_from: Option<String>,
    },

    #[returns(Vec<crate::minter::types::CurrencyInfo>)]
    CurrencyInfoListByOwner {
        owner: String,
        amount: u32,
        start_from: Option<String>,
    },

    #[returns(Vec<(Addr, u16)>)]
    TokenCountList {
        amount: u32,
        start_from: Option<String>,
    },
}
