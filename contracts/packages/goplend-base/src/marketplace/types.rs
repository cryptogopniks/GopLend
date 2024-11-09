use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub lending_platform: Addr,
    pub oracle: Addr,
    pub marketplace: Addr,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
