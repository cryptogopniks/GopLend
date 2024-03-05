use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::{
    assets::{Currency, Token, TokenUnverified},
    minter::types::Metadata,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
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
    },

    CreateCw20 {
        token_owner: String,
        name: String,
        symbol: String,
        decimals: u8,
        marketing: Option<cw20_base::msg::InstantiateMarketingInfo>,
    },

    Mint {
        token: TokenUnverified,
        amount: Uint128,
        recipient: String,
    },

    Burn {},

    SetMetadataNative {
        metadata: Metadata,
    },

    UpdateMarketingCw20 {
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    },

    UploadLogoCw20(cw20::Logo),

    ChangeAdminNative {
        denom: String,
    },

    UpdateMinterCw20 {
        new_minter: Option<String>,
    },

    // admin
    RegisterCurrency {
        currency: Currency<Token>,
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
    CurrenciesByCreator { creator: String },

    #[returns(crate::minter::types::Config)]
    QueryConfig {},
}
