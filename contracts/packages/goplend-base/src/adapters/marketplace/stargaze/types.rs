use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::assets::{Currency, Token};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub lending_platform: Option<Addr>,
    pub oracle: Option<Addr>,
    pub marketplace: Option<Addr>,
    pub dex: Option<Addr>,
    pub main_currency: Currency<Token>,
    pub price_update_period: u64,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
