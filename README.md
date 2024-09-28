### Project Description

`GopLend` is a lending protocol allowing users to borrow tokens (`STARS` or `USDC`) against blue chip NFTs. The application provides the following features:

1) Depositing `STARS` on a floating rate deposit with a 2 week unbonding period. This can be a good alternative to `STARS` staking - with a fixed borrowing rate of 25 % and a platform fee of 5 %, depositors can expect a 20 % APR with an equal liquidity ratio of depositors to borrowers, which exceeds the APR of staking (16 % at the time of writing). Moreover, the profitability of deposits is increased due to the funds received from the liquidation of collateral.
Upon opening a deposit, users receive `bglSTARS` (bonded GopLend STARS) token in return, which reflect their share of ownership of a common `STARS` pool on the platform. Over time, this pool will increase due to borrower payments, while the number of `bglSTARS` will remain unchanged, allowing the latter to grow in value relative to `STARS`, accumulating the platform's revenue.

2) Arbitrage of `bglSTARS`. Users will be able to exchange `bglSTARS` for `STARS` on DEX, avoiding the unbonding period or to buy `bglSTARS` with a small discount. The latter option allows not only a more profitable way to become a depositor, but also opens up opportunities for earning on arbitrage, when users can earn by buying `bglSTARS` on DEX and converting it to `STARS` on the platform during the unbonding period.

3) `STARS` loan secured by NFT. Borrowers have clear and convenient conditions - an open-ended loan in `STARS` with a fixed rate (25 %), you can borrow up to 65 % of the pledge amount, liquidation occurs when LTV (loan to value ratio) increases to 72.7 %. The value of the collateral is calculated taking into account the price of offers, volume of offers, trading volume, which allows to effectively smooth price fluctuations and protect borrowers from liquidation due to market volatility.

4) Collateral Liquidation Bids. Default source of liquidation offers is the Marketplace however if users bids exist on the platform, they will be prioritized. Users have a great opportunity to purchase NFTs at a discount by placing bids in `STARS` at a certain discount to liquidate specified NFTs. Bids with a smaller discount, narrower coverage and earlier creation date will be prioritized for liquidation.

5) Stablecoin Market. Despite the fact that STARS is the main currency of Stargaze and the network has no exchanges, GopLend has a separate platform where all transactions are done in USDC (and the receipt token is bglUSDC). This is a great option for users who do not wish to deal with volatile currencies. It should be taken into account that the value of NFTs in stablecoins is 3% lower than in the main currency, due to conversion losses

6) Proof of Ownership. When a user holds a NFT on his account he can claim airdrops; confirm token ownership in Discord to get roles; track their assets in the portfolio tracker. The GopLend team believes that owning a token in a contract is no different than owning a token on regular account, so all of these privileges should be maintained. The app allows to make snapshots of collateral ownership including ownership start date that can be easily requested by any app/project

7) Telegram Bots. When you are leveraged, it is important to be able to receive notifications of LTV changes so you can react in time to avoid liquidation. Depositors may be interested in information about current deposit rates, while arbitrageurs may be interested in the price of returnable tokens on exchanges. The GopLend team plans to develop a free Telegram bot with this functionality

### Listing Process

Any NFT collection must be registered (listed), for which there is a mechanism for dealing with proposals. The process is as follows:
1. The collection creator passes the collection details to the GopLend team
2. The team verifies the data, decides on the listing, and sets the listing price (usually 12 collection floor prices to replenish platforms liquidity pools). It's important to list only high quality (blue chip) collections to minimize risks for users and don't increase unreasonably the load on the app infrastructure. That's why the collection must meet the list of requirements such as lifetime, trading volume, community engagement, etc.
3. The GopLend creates a governance proposal via CryptoGopniks DAO to add a new collection. GOP token stakers can vote for listing. If the proposal will reach the quorum with majority of positive votes the collection will be listed

### Deposits

When a user makes a deposit to `STARS`, `bglSTARS` is minted to his address in the amount of
```
bglstars_amount = stars_amount * bglstars_supply / stars_pool_volume
```

The price of `bglSTARS` on the platform is defined as
```
bglstars_price = stars_price * stars_pool_volume / bglstars_supply
```

The price of `bglSTARS` in pools on DEX will also follow this formula, adjusted by arbitrage.

Initially the price of `bglSTARS` is equal to the price of `STARS`, but as the pool grows, it will increase. For example, a user deposited 1000 `STARS` and received 1000 `bglSTARS` in return. After some time the pool grew by 20 %. The user made `Unbond` and burned 500 `bglSTARS` on the platform. In 2 weeks he will be able to collect 600 `STARS`.

The yield on deposit is calculated as
```
deposit_apr = (BORROW_APR - BORROW_FEE_RATE) * total_borrowed_amount / (total_deposited_amount - total_unbonded_amount)
```
where `BORROW_APR` is 25 %, `BORROW_FEE_RATE` is 5 %.

`total_borrowed_amount` may exceed `total_deposited_amount`, as the `STARS` pool is replenished from borrower payments and income from sales of collateral assets.

`Deposit` action instantly decreases `deposit_apr`. `Unbond` instantly increases `deposit_apr` without withdrawing liquidity from the `STARS` pool. `Withdraw` does not affect `deposit_apr`, but withdraws liquidity from the `STARS` pool.

### Loans

The borrower can add collateral using `DepositCollateral` message by specifying the desired collections and their tokens. This action reduces LTV. The opposite action is `WithdrawCollateral`, it cannot be performed if it will increase current LTV to >= 65 %. `Repay` and `Borrow` affect the size of the loan and work similarly with respect to LTV.

The collateral price is calculated as follows:
1) The arithmetic average of the prices of the last 5 offers is calculated. If the total number of offers is less than 5, the missing values are filled with zeros
```
average_offer_price = sum_over_last_5_offers(max(offer_price, 0)) / 5
```
2) The liquidity ratio is defined as
```
liquidity_multiplier = min(trading_volume_per_day / (10 * average_offer_price), 1)
```
3) Total value
```
collateral_price = average_offer_price * liquidity_multiplier
```

### Liquidations

A liquidation occurs when the on-chain scheduler calls the `lending-platform` contract with a `Process` message when the borrower's LTV value is at least `LTV_MAX`.

To avoid liquidations due to `oracle` failures, the `lending-platform` contract must check the update date of prices received from `oracle`. New prices are written to `oracle` every hour. If the prices received from `oracle` were updated more than one hour ago, such data is considered invalid and the corresponding collections should be ignored by the liquidation logic in the current invocation of the `lending-platform` contract.

At the time of liquidation, the collateral price must be adequate in relation to normal offers in order to make it profitable for liquidators to bid on the `lending-platform`. The minimum discounted pledge price should be on par with average or high offers. The value of the collateral with the maximum discount should be such that it is possible to WARRANTEE the sale of the collateral (the level of the lowest offers).

If there are no bids from liquidators, the platform will liquidate the token itself, accepting the best offer to get liquidity quickly.

If there are bids, first a check for sufficiency of funds is made; then those with the lowest discount are selected; then those that target specific tokens, collections, general (in descending order of priority); then those with an earlier last change date. If a liquidator has a bid for multiple tokens from the same collection, they will be liquidated in the order in which they were listed when the bid was placed.

The `LTV_MAX`, `BID_MIN`, `BID_MAX` parameters are calculated based on the following considerations. The platform takes a fee of 5% for any liquidation (`LIQUIDATION_FEE_RATE`). If the deposit is liquidated with the maximum discount, the remaining part of the profit (5%) goes to the common `STARS` pool, increasing the profitability of deposits, i.e.
```
BID_MIN = 1.1 * loan
```
In order to offer attractive terms to liquidators, you need to have a wide range of discount. In this case it ranges from 0 to 20 %. From here the maximum bid (with the minimum discount) is determined
```
BID_MAX = BID_MIN / (1 - (DISCOUNT_MAX - DISCOUNT_MIN))
BID_MAX = 1.375 * loan
```
From here it is easy to get the `LTV_MAX` value
```
LTV_MAX = loan / BID_MAX
LTV_MAX = 0.727
```

Thus, at the maximum discount, the collateral is redeemed for 110 % of the loan value and 5 % of the profit goes to the `STARS` pool, and at the minimum discount, the collateral is redeemed for 137.5 % of the loan value and 32.5 % of the profit goes to the `STARS` pool. The discount itself varies from 0 to 20 %.

### Market Maker

Market Maker is a separate app (2 contracts + 1 script) handling collection offers for listed collections. It allows to keep 10 offers in lower quarter of allowed offers range to stabilize collection price and provide more liquidity for liquidations. When amount of offers in allowed range drop to 5 it update out of range offers to get 10 proper offers again

To enable MM for new collection admin or worker must list it on MM specifying collection owners. Then one of these owners must deposit liquidity that will be used to put offers. It's recommended to provide liquidity in amout of 17 * `collection_floor_price` to have enough liquidity to update offers when collection price will go up
