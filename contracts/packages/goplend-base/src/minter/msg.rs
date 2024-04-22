use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use cw20::Logo;
use cw20_base::msg::InstantiateMarketingInfo;

use crate::{
    assets::{Currency, TokenUnverified},
    minter::types::Metadata,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
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
        marketing: Option<InstantiateMarketingInfo>,
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
        logo: Logo,
    },

    ChangeAdminNative {
        token: TokenUnverified,
    },

    UpdateMinterCw20 {
        token: TokenUnverified,
        new_minter: String,
    },

    // any
    AcceptAdminRole {},

    // admin
    RegisterCurrency {
        currency: Currency<TokenUnverified>,
        creator: String,
    },

    UpdateConfig {
        admin: Option<String>,
        worker: Option<String>,
        cw20_base_code_id: Option<u64>,
    },

    UpdateTokenOwner {
        denom: String,
        owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::minter::types::QueryCurrenciesFromCreatorResponse)]
    QueryCurrenciesByCreator { creator: String },

    #[returns(crate::minter::types::Config)]
    QueryConfig {},

    #[returns(cosmwasm_std::Addr)]
    QueryTokenOwner { denom: String },
}
