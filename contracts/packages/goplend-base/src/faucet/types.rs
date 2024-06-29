use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub claimable_amount: Uint128,
    pub claim_cooldown: u64,
}
