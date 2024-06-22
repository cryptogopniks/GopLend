use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};

#[cw_serde]
pub enum Route {
    Neutron,
    Osmosis,
    Test,
    Test2,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub lending_platform: Option<Addr>,
    pub adapter_marketplace: Option<Addr>,
    pub recover_address: Option<Addr>,
    pub max_price_impact: Decimal,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
