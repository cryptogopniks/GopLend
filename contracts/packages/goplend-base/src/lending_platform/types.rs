use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::assets::{Currency, Funds, Token};

#[cw_serde]
pub struct Collection<A: ToString> {
    pub name: String,
    pub owner: A,
}

#[cw_serde]
pub struct Proposal<A: ToString, T: From<Token>> {
    pub proposal_status: Option<ProposalStatus>,
    pub proposal_type: ProposalType<A>,
    pub listing_price: Funds<T>,
}

#[cw_serde]
pub enum ProposalStatus {
    Active,
    Accepted,
    Rejected,
}

#[cw_serde]
pub enum ProposalType<A: ToString> {
    AddCollection {
        collection_address: A,
        collection: Collection<A>,
    },
    UpdateCollection {
        collection_address: A,
        new_collection_address: Option<A>,
        new_collection: Collection<A>,
    },
}

#[cw_serde]
pub struct CollectionInfo<A: ToString> {
    pub collection_address: A,
    pub token_id_list: Vec<String>,
}

#[cw_serde]
pub struct LiquidationBid {
    pub creation_date: u64,
    pub liquidator: Addr,
    pub token_info_list: Vec<String>,
    pub amount: Uint128,
    pub discount: Decimal,
}

#[cw_serde]
pub struct Collateral {
    pub owner: Addr,
    pub token_id_list: Vec<String>,
}

#[cw_serde]
pub struct Vault {
    pub creation_date: u64,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Depositor {
    pub unbonded: Vec<Vault>,
}

#[cw_serde]
pub struct Borrower {
    pub accumulated_loan: Uint128, // used for merging vaults
    pub loan: Vault,
}

#[cw_serde]
pub struct Liquidator {
    pub collection_addresses: Vec<Addr>,
}

#[derive(Default)]
#[cw_serde]
pub struct Balances {
    pub total_deposited: Uint128,
    pub total_unbonded: Uint128,
    pub total_borrowed: Uint128,
    pub total_liquidation_bids: Uint128,
    pub total_main_currency_pool: Uint128,
    pub total_bgl_currency_supply: Uint128,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub owner: Option<Addr>,
    pub oracle: Option<Addr>,
    pub main_currency: Currency<Token>,
    pub bgl_currency: Option<Currency<Token>>,
    pub cw20_base_code_id: u64,
    pub unbonding_period: u64,
    pub borrow_apr: Decimal,
    pub borrow_fee_rate: Decimal,
    pub liquidation_fee_rate: Decimal,
    pub discount_min_rate: Decimal,
    pub discount_max_rate: Decimal,
    pub bid_min_rate: Decimal,
}
