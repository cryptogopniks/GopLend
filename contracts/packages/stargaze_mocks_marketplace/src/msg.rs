use cosmwasm_schema::cw_serde;

use crate::state::Bid;

#[cw_serde]
pub struct InstantiateMsg {
    pub bids: Vec<Bid>,
}
