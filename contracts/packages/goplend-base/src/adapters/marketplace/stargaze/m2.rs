use cosmwasm_std::{Addr, Coin};
use cw_address_like::AddressLike;

enum QueryBound<T> {
    Inclusive(T),
    Exclusive(T),
}

struct QueryOptions<T> {
    /// The number of items that will be returned
    pub limit: Option<u32>,
    /// Whether to sort items in ascending or descending order
    pub descending: Option<bool>,
    /// The minimum key value to fetch
    pub min: Option<QueryBound<T>>,
    /// The maximum key value to fetch
    pub max: Option<QueryBound<T>>,
}

pub type Denom = String;
pub type OrderId = String;
pub type TokenId = String;

struct OrderDetails<T: AddressLike> {
    pub price: Coin,
    pub recipient: Option<T>,
    pub finder: Option<T>,
}

struct PriceOffset {
    pub id: OrderId,
    pub amount: u128,
}

enum ExecuteMsg {
    AcceptCollectionBid {
        id: OrderId,
        token_id: TokenId,
        details: OrderDetails<String>,
    },

    AcceptBid {
        id: OrderId,
        details: OrderDetails<String>,
    },

    SetCollectionBid {
        collection: String,
        details: OrderDetails<String>,
    },

    RemoveCollectionBid {
        id: OrderId,
    },
}

enum QueryMsg {
    // Vec<Ask>
    AsksByCollectionDenom {
        collection: String,
        denom: Denom,
        query_options: Option<QueryOptions<PriceOffset>>,
    },

    // Vec<Bid>
    BidsByTokenPrice {
        collection: String,
        token_id: TokenId,
        denom: Denom,
        query_options: Option<QueryOptions<PriceOffset>>,
    },

    // Params {}?

    // TODO: probably not
    // Vec<Bid>
    BidsByCreatorCollection {
        creator: String,
        collection: String,
        query_options: Option<QueryOptions<String>>,
    },

    // Vec<CollectionBid>
    CollectionBidsByPrice {
        collection: String,
        denom: Denom,
        query_options: Option<QueryOptions<PriceOffset>>,
    },

    // Option<CollectionBid>
    CollectionBid(String),
}

// responses

struct Ask {
    pub id: String,
    pub creator: Addr,
    pub collection: Addr,
    pub token_id: TokenId,
    pub details: OrderDetails<Addr>,
}

struct CollectionBid {
    pub id: String,
    pub creator: Addr,
    pub collection: Addr,
    pub details: OrderDetails<Addr>,
}

struct Bid {
    pub id: String,
    pub creator: Addr,
    pub collection: Addr,
    pub token_id: TokenId,
    pub details: OrderDetails<Addr>,
}
