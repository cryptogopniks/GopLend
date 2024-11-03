use cosmwasm_std::{Addr, Timestamp, Uint128};

pub type Bidder = String;
pub type Collection = String;
pub type TokenId = u32;

struct AskOffset {
    pub price: Uint128,
    pub token_id: TokenId,
}

struct BidOffset {
    pub price: Uint128,
    pub token_id: TokenId,
    pub bidder: Addr,
}

struct CollectionBidOffset {
    pub price: Uint128,
    pub collection: Collection,
    pub bidder: Bidder,
}

enum ExecuteMsg {
    AcceptCollectionBid {
        collection: String,
        token_id: TokenId,
        bidder: String,
        amount: Uint128,
        finder: Option<String>,
    },

    AcceptBid {
        collection: String,
        token_id: TokenId,
        bidder: String,
        amount: Uint128,
        finder: Option<String>,
    },

    SetCollectionBid {
        collection: String,
        expires: Timestamp,
        finders_fee_bps: Option<u64>,
    },

    RemoveCollectionBid {
        collection: String,
    },
}

enum QueryMsg {
    // AsksResponse {
    //     pub asks: Vec<Ask>,
    // }
    AsksSortedByPrice {
        collection: Collection,
        include_inactive: Option<bool>,
        start_after: Option<AskOffset>,
        limit: Option<u32>,
    },

    // BidsResponse {
    //     pub bids: Vec<Bid>,
    // }
    Bids {
        collection: Collection,
        token_id: TokenId,
        start_after: Option<Bidder>,
        limit: Option<u32>,
    },

    // ParamsResponse {
    //     pub params: SudoParams,
    // }
    Params {},

    // BidsResponse {
    //     pub bids: Vec<Bid>,
    // }
    ReverseBidsSortedByPrice {
        collection: Collection,
        start_before: Option<BidOffset>,
        limit: Option<u32>,
    },

    // CollectionBidsResponse {
    //     pub bids: Vec<CollectionBid>,
    // }
    ReverseCollectionBidsSortedByPrice {
        collection: Collection,
        start_before: Option<CollectionBidOffset>,
        limit: Option<u32>,
    },

    // CollectionBidResponse {
    //     pub bid: Option<CollectionBid>,
    // }
    CollectionBid {
        collection: Collection,
        bidder: Bidder,
    },
}
