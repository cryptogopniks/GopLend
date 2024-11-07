use std::cmp::Ordering;

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
    pub token_id_list: Vec<String>,
    pub amount: Uint128,
    pub discount: Decimal,
    pub bid_type: BidType,
    /// only for CollectionOffer and TokenOffer
    pub order_id: Option<String>,
}

#[derive(Eq)]
#[cw_serde]
pub enum BidType {
    LiquidationBid,
    CollectionOffer,
    TokenOffer,
}

// bids.sort_unstable_by_key(|(_, bid)| bid.bid_type) will give
// [LiquidationBid, TokenOffer, CollectionOffer]
impl Ord for BidType {
    fn cmp(&self, other: &BidType) -> Ordering {
        match (self, other) {
            (BidType::LiquidationBid, BidType::LiquidationBid) => Ordering::Equal,
            (BidType::LiquidationBid, _) => Ordering::Less,
            (BidType::TokenOffer, BidType::TokenOffer) => Ordering::Equal,
            (BidType::TokenOffer, BidType::LiquidationBid) => Ordering::Greater,
            (BidType::TokenOffer, BidType::CollectionOffer) => Ordering::Less,
            (BidType::CollectionOffer, BidType::CollectionOffer) => Ordering::Equal,
            (BidType::CollectionOffer, _) => Ordering::Greater,
        }
    }
}

impl PartialOrd for BidType {
    fn partial_cmp(&self, other: &BidType) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cw_serde]
pub struct Collateral {
    pub owner: Addr,
    pub token_item_list: Vec<TokenItem>,
}

#[cw_serde]
pub struct TokenItem {
    pub id: String,
    pub creation_date: u64, // enables holding snapshots functionality
}

#[cw_serde]
pub struct LiquidationItem {
    pub borrower: String,
    pub liquidation_set: Vec<BiddedCollateralItem>,
}

#[cw_serde]
pub struct BiddedCollateralItem {
    pub collection: Addr,
    pub token_item: TokenItem,
    pub owner: Addr,
    pub liquidator: Addr,
    pub bid_creation_date: u64,
    pub liquidation_price: Uint128,
    pub collateral_price: Uint128,
    pub bid_amount: Uint128,
    pub bid_discount: Decimal,
    pub bid_type: BidType,
    pub order_id: String,
}

#[derive(Default)]
#[cw_serde]
pub struct Vault {
    /// used to calculate accumulated rewards before changing amount
    pub creation_date: u64,
    pub amount: Uint128,
}

#[derive(Default)]
#[cw_serde]
pub struct Unbonder {
    pub returned_and_unbonded: Vec<(Uint128, Vault)>,
}

#[derive(Default)]
#[cw_serde]
pub struct Borrower {
    /// used for merging vaults
    pub accumulated_loan: Uint128,
    pub loan: Vault,
}

#[derive(Default)]
#[cw_serde]
pub struct Liquidator {
    pub collection_addresses: Vec<Addr>,
}

#[derive(Default)]
#[cw_serde]
pub struct Balances {
    pub regular: RegularBalances,
    pub reserve: ReserveBalances,
}

#[derive(Default)]
#[cw_serde]
pub struct RegularBalances {
    /// increased on try_deposit                                                                    \
    /// decreased on try_withdraw   
    pub deposited: Uint128,
    /// increased on try_unbond                                                                     \
    /// decreased on try_withdraw   
    pub unbonded: Uint128,
    /// increased on try_borrow if regular.deposited - regular.unbonded - regular.borrowed > 0      \
    /// decreased on try_repay, try_liquidate if reserve.borrowed == 0
    pub borrowed: Vault,

    /// used to match all loans by creation date
    pub accumulated_rewards: Uint128,
    pub liquidation_bids: Uint128,

    /// used to determine bgl price and deposit apr                                                 \
    /// increased on try_deposit - full; try_repay, try_liquidate - rewards                         \
    /// decreased on try_withdraw
    pub main_pool: Uint128,
    /// increased on try_deposit                                                                    \
    /// decreased on try_withdraw
    pub bgl_supply: Uint128,
}

#[derive(Default)]
#[cw_serde]
pub struct ReserveBalances {
    /// increased on try_deposit_reserve_liquidity                                                  \
    /// decreased on try_withdraw_reserve_liquidity
    pub deposited: Uint128,

    /// used to provide temporary liquidity to withdrawers w/o affecting on bgl price and deposit apr        \
    /// has higher priority than reserve.borrowed                                                    \
    /// increased on try_withdraw if withdrawal_amount > regular.main_pool                           \
    /// decreased on try_repay, try_liquidate if reserve.withdrawn > 0
    pub withdrawn: Uint128,
    /// used to provide temporary liquidity to borrowers w/o affecting on bgl price and deposit apr        \
    /// accumulated rewards goes to regular part as reserve liquidity provider can't earn           \
    /// increased on try_borrow if regular.deposited - regular.unbonded - regular.borrowed == 0     \
    /// decreased on try_repay, try_liquidate if reserve.borrowed > 0
    pub borrowed: Vault,
}

#[cw_serde]
pub struct AddressConfig {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub oracle: Option<Addr>,
    pub minter: Option<Addr>,
    pub adapter_scheduler: Option<Addr>,
    pub adapter_marketplace: Option<Addr>,
}

#[cw_serde]
pub struct RateConfig {
    pub borrow_apr: Decimal,
    pub borrow_fee_rate: Decimal,
    pub liquidation_fee_rate: Decimal,
    pub discount_min_rate: Decimal,
    pub discount_max_rate: Decimal,
    pub bid_min_rate: Decimal,
}

#[cw_serde]
pub struct CommonConfig {
    pub main_currency: Currency<Token>,
    pub bgl_currency: Option<Currency<Token>>,
    pub unbonding_period: u64,
    pub collateral_min_value: Uint128,
    pub borrowers_reserve_fraction_ratio: Decimal,
    pub is_marketplace_interface_enabled: bool,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
