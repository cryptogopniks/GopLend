use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::{
    assets::{Currency, Token, TokenUnverified},
    minter::types::Metadata,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub cw20_base_code_id: Option<u64>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(cw20::Cw20ReceiveMsg),

    CreateNative {
        token_owner: String,
        subdenom: String,
        decimals: Option<u8>,
    },

    CreateCw20 {
        token_owner: String,
        name: String,
        symbol: String,
        decimals: Option<u8>,
        marketing: Option<cw20_base::msg::InstantiateMarketingInfo>,
    },

    Mint {
        token: TokenUnverified,
        amount: Uint128,
        recipient: String,
    },

    Burn {},

    SetMetadataNative {
        token: TokenUnverified,
        metadata: Metadata,
    },

    UpdateMarketingCw20 {
        token: TokenUnverified,
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    },

    UploadLogoCw20 {
        token: TokenUnverified,
        logo: cw20::Logo,
    },

    ChangeAdminNative {
        token: TokenUnverified,
    },

    UpdateMinterCw20 {
        token: TokenUnverified,
        new_minter: Option<String>,
    },

    // admin
    RegisterCurrency {
        currency: Currency<TokenUnverified>,
        creator: String,
    },

    UpdateConfig {
        owner: Option<String>,
        cw20_base_code_id: Option<u64>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::minter::types::QueryCurrenciesFromCreatorResponse)]
    QueryCurrenciesByCreator { creator: String },

    #[returns(crate::minter::types::Config)]
    QueryConfig {},
}
