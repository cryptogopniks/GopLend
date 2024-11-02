export type AddressLike = Addr | string;
export type Addr = string;
export type Denom = string;
export type OrderId = string;
export type TokenId = string;
export type u32 = number;
export type u128 = number;
export type Uint128 = string;

export type QueryBound<T> = { inclusive: T } | { exclusive: T };

export interface Coin {
  denom: string;
  amount: Uint128;
}

export interface QueryOptions<T> {
  limit?: u32;
  descending?: boolean;
  min?: QueryBound<T>;
  max?: QueryBound<T>;
}

export interface OrderDetails<T> {
  price: Coin;
  recipient?: T;
  finder?: T;
}

export interface PriceOffset {
  id: OrderId;
  amount: u128;
}

export interface QueryAsksByCollectionDenom {
  asks_by_collection_denom: {
    collection: string;
    denom: Denom;
    query_options?: QueryOptions<PriceOffset>;
  };
}

export interface QueryBidsByTokenPrice {
  bids_by_token_price: {
    collection: string;
    token_id: TokenId;
    denom: Denom;
    query_options?: QueryOptions<PriceOffset>;
  };
}

export interface QueryBidsByCreatorCollection {
  bids_by_creator_collection: {
    creator: string;
    collection: string;
    query_options?: QueryOptions<string>;
  };
}

export interface QueryCollectionBidsByPrice {
  collection_bids_by_price: {
    collection: string;
    denom: Denom;
    query_options?: QueryOptions<PriceOffset>;
  };
}

export interface CollectionBid {
  id: string;
  creator: Addr;
  collection: Addr;
  details: OrderDetails<Addr>;
}

export interface Ask {
  id: string;
  creator: Addr;
  collection: Addr;
  token_id: TokenId;
  details: OrderDetails<Addr>;
}

export interface Bid {
  id: string;
  creator: Addr;
  collection: Addr;
  token_id: TokenId;
  details: OrderDetails<Addr>;
}

export interface QueryCollectionBid {
  collection_bid: string;
}
export type QueryCollectionBidResponse = CollectionBid | undefined;
