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
    pub worker: Option<Addr>,
    pub lending_platform_main: Option<Addr>,
    pub lending_platform_stable: Option<Addr>,
    pub offchain_clock: Option<Vec<Addr>>,
    pub clock_source: ClockSource,
    pub execution_cooldown: u64,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}

#[cw_serde]
pub struct Log {
    pub block_time: u64,
    pub sender: Addr,
    pub counter: u8,
}
