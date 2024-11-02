use cosmwasm_std::Coin;
use cw_address_like::AddressLike;
use sg_index_query::QueryOptions;

pub type Denom = String;
pub type OrderId = String;
pub type TokenId = String;

pub struct OrderDetails<T: AddressLike> {
    pub price: Coin,
    pub recipient: Option<T>,
    pub finder: Option<T>,
}

pub struct PriceOffset {
    pub id: OrderId,
    pub amount: u128,
}

pub enum ExecuteMsg {
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

pub enum QueryMsg {
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
