use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::assets::{Currency, Token};

#[cw_serde]
pub struct CurrencyInfo {
    pub currency: Currency<Token>,
    /// for cw20 based tokens
    pub cw20_code_id: Option<u64>,
    /// can update the token CurrencyInfo and FaucetConfig
    pub owner: Addr,
    /// can mint/burn
    pub whitelist: Vec<Addr>,
    /// if true any token holder can burn his tokens in minter
    pub permissionless_burning: bool,
}

#[cw_serde]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

#[cw_serde]
pub struct Metadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: Option<String>,
    pub uri_hash: Option<String>,
}

#[cw_serde]
pub struct Config {
    /// can update Config
    pub admin: Addr,
    /// can create and register tokens
    pub whitelist: Vec<Addr>,
    pub cw20_code_id: Option<u64>,
    pub permissionless_token_creation: bool,
    pub permissionless_token_registration: bool,
    /// max amount of tokens for non-whitelisted owner
    pub max_tokens_per_owner: u16,
}

#[derive(Default)]
#[cw_serde]
pub struct FaucetConfig {
    /// if zero then faucet is disabled
    pub claimable_amount: Uint128,
    /// in seconds
    pub claim_cooldown: u64,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
