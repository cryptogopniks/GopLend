use cosmwasm_schema::cw_serde;

use crate::state::Bid;

#[cw_serde]
pub struct InstantiateMsg {
    pub bids: Vec<Bid>,
}

#[cw_serde]
pub enum SudoMsg {
    SetBids { bids: Vec<Bid> },

    SetRoyaltyRegistry(String),
}
