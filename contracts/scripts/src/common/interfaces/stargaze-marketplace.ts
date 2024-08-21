export type Duration = { height: number } | { time: number };

export interface ExpiryRange {
  min: number;
  max: number;
}

export interface QueryParams {
  params: {};
}

export interface ParamsResponse {
  params: {
    // Fair Burn fee for winning bids
    trading_fee_percent: String;
    // Valid time range for Asks
    // (min, max) in seconds
    ask_expiry: ExpiryRange;
    // Valid time range for Bids
    // (min, max) in seconds
    bid_expiry: ExpiryRange;
    // Operators are entites that are responsible for maintaining the active state of Asks
    // They listen to NFT transfer events, and update the active state of Asks
    operators: string[];
    // Max value for the finders fee
    max_finders_fee_percent: string;
    // Min value for a bid
    min_price: string;
    // Duration after expiry when a bid becomes stale
    stale_bid_duration: Duration;
    // Stale bid removal reward
    bid_removal_reward_percent: string;
    // Listing fee to reduce spam
    listing_fee: string;
  };
}

export interface QueryCollectionInfo {
  collection_info: {};
}

export interface RoyaltyInfoResponse {
  payment_address: string;
  share: string;
}

export interface Timestamp {
  timestamp: string;
}

export interface CollectionInfoResponse {
  creator: String;
  description: String;
  image: String;
  external_link?: string;
  explicit_content?: boolean;
  start_trading_time?: Timestamp;
  royalty_info?: RoyaltyInfoResponse;
}

export interface QueryReverseBidsSortedByPrice {
  reverse_bids_sorted_by_price: {
    collection: string;
    start_before?: BidOffset;
    limit?: number;
  };
}

export interface BidOffset {
  bidder: string;
  price: string;
  token_id: number;
}

export interface Bid {
  collection: string;
  token_id: number;
  bidder: string;
  price: string;
  finders_fee_bps?: number;
  expires_at: string;
}

export interface BidsResponse {
  bids: Bid[];
}

export interface CollectionBidOffset {
  price: string;
  collection: string;
  bidder: string;
}

export interface QueryReverseCollectionBidsSortedByPrice {
  reverse_collection_bids_sorted_by_price: {
    collection: string;
    start_before?: CollectionBidOffset;
    limit?: number;
  };
}

export interface CollectionBid {
  collection: string;
  bidder: string;
  price: string;
  finders_fee_bps?: number;
  expires_at: string;
}

export interface CollectionBidsResponse {
  bids: CollectionBid[];
}
