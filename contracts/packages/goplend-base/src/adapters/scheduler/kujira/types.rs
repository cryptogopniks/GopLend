use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub enum ClockSource {
    Onchain,
    Offchain,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub lending_platform: Option<Addr>,
    pub offchain_clock: Option<Vec<Addr>>,
    pub clock_source: ClockSource,
    pub min_clock_period: u64,
}
