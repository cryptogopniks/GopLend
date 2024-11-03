use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub enum PlatformType {
    Main,
    Stable,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub lending_platform_main: Option<Addr>,
    pub lending_platform_stable: Option<Addr>,
    pub oracle: Option<Addr>,
    pub marketplace: Option<Addr>,
    pub adapter_dex: Option<Addr>,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
