use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};

#[cw_serde]
pub enum Route {
    Empty,
    Neutron,
    Osmosis,
    Test,
}

#[cw_serde]
pub enum IbcMemo {
    Forward {
        channel: String,
        port: String,
        receiver: String,
        retries: u8,
        timeout: u64,
    },
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub lending_platform_stable: Option<Addr>,
    pub adapter_marketplace: Option<Addr>,
    pub recover_address: Option<Addr>,
    pub max_price_impact: Decimal,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
