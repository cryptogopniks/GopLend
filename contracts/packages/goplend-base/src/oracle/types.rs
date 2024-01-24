use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub owner: Option<Addr>,
    pub lending_platform: Option<Addr>,
}

impl Config {
    pub fn new(admin: &Addr, owner: &Option<Addr>, lending_platform: &Option<Addr>) -> Self {
        Self {
            admin: admin.to_owned(),
            owner: owner.to_owned(),
            lending_platform: lending_platform.to_owned(),
        }
    }
}
