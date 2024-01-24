use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub owner: Option<Addr>,
    pub oracle: Option<Addr>,
}

impl Config {
    pub fn new(admin: &Addr, owner: &Option<Addr>, oracle: &Option<Addr>) -> Self {
        Self {
            admin: admin.to_owned(),
            owner: owner.to_owned(),
            oracle: oracle.to_owned(),
        }
    }
}
