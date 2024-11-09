use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub lending_platform: Addr,
    pub offchain_clock: Vec<Addr>,
    pub execution_cooldown: u64,
}

#[cw_serde]
pub struct Log {
    pub block_time: u64,
    pub sender: Addr,
    pub counter: u8,
}
