### Project description

`GopLend` is a lending platform that uses NFTs as collateral. The application provides the following features:

1) Depositing `STARS` on a floating rate deposit with a 2 week unbonding period. This can be a good alternative to `STARS` staking - with a fixed borrowing rate of 25 % and a platform fee of 5 %, depositors can expect a 20 % APR with an equal liquidity ratio of depositors to borrowers, which exceeds the APR of staking (15.6 % at the time of writing). Moreover, the profitability of deposits is increased due to the funds received from the liquidation of collateral.
Upon opening a deposit, users receive `bglSTARS` (bonded GopLend `STARS`) CW20 token in return, which reflect their share of ownership of a common `STARS` pool on the platform. Over time, this pool will increase due to borrower payments, while the number of `bglSTARS` will remain unchanged, allowing the latter to grow in value relative to `STARS`, accumulating the platform's revenue.

2) Arbitrage of `bglSTARS`. If there are liquidity pools on DEX, some users will be able to exchange `bglSTARS` for `STARS`, avoiding the anbonding period, and others will be able to buy `bglSTARS` with a small discount. The latter option allows not only a more profitable way to become a depositor, but also opens up opportunities for earning on arbitrage, when users can earn by buying `bglSTARS` on DEX and converting it to `STARS` on the platform during the rebonding period.

3) `STARS` loan secured by NFT. Borrowers have clear and convenient conditions - an open-ended loan in `STARS` with a fixed rate (25 %), you can borrow up to 65.4 % of the pledge amount, liquidation occurs when LTV (loan to value ratio) increases to 72.7 %. The value of the collateral is calculated taking into account the price of offers, volume of offers, trading volume, which allows to effectively smooth price fluctuations and protect borrowers from liquidation due to market volatility.

4) Collateral Liquidation Bids. Users have a great opportunity to purchase NFTs at a discount by placing bids in `STARS` at a certain discount to liquidate any NFT, selected collections or specific NFTs. Bids with a smaller discount, narrower coverage and earlier creation date will be prioritized for liquidation.

The application consists of two contracts: `lending-platform` and `oracle`. The first one contains main logic while the second one is required to provide on-chain NFT collections price data. In addition an off-chain scheduler is involved to write data to the `oracle` contract, and an on-chain scheduler to trigger collateral liquidation logic in the `lending-platform` contract.

Any NFT collection must be registered (listed), for which there is a mechanism for dealing with proposals. The process is as follows:
1. The collection creator passes the collection details to the `CryptoGopniks` team
2. The team verifies the data, decides on the listing, and sets the listing price
3. The admin or the owner of the contract (saved in the contract config by the admin) creates a proposal to add a new collection. The proposal becomes active
4. The creator of the collection (specified in the proposal as collection owner) can accept the proposal by signing a transaction with the required amount. In this case, the status of the proposal will change to "accepted" and the deposited amount will be sent to a special balance, from where the admin/owner can distribute it to the required list of addresses with the specified weights. If necessary, admin/owner can reject the active collection (status will change to "rejected") and create a new one with changed parameters. Also an erroneously registered collection can be deleted by admin/owner

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

Initially the price of `bglSTARS` is equal to the price of `STARS`, but as the pool grows, it will increase. For example, a user deposited 1000 `STARS` and received 1000 `bglSTARS` in return. After some time the pool grew by 20 %. The user made Unbond and burned 500 `bglSTARS` on the platform. In 2 weeks he will be able to collect 600 `STARS`.

The yield on deposit is calculated as
```
deposit_apr = (BORROW_APR - BORROW_COMMISION_RATE) * total_borrowed_amount / total_deposited_amount
```
where `BORROW_APR` is 25 %, `BORROW_COMMISION_RATE` is 5 %.

`total_borrowed_amount` may exceed `total_deposited_amount`, as the `STARS` pool is replenished from borrower payments and income from sales of collateral assets.

`Deposit` message instantly decreases `deposit_apr`. `Unbond` instantly increases `deposit_apr` without withdrawing liquidity from the `STARS` pool. `Withdraw` does not affect `deposit_apr`, but withdraws liquidity from the `STARS` pool.

### Loans

The borrower can add collateral using `DepositCollateral` message by specifying the desired collections and their tokens. This action reduces LTV. The opposite action is `WithdrawCollateral`, it cannot be performed if it will increase current LTV to >= 65.4 % (90 % of the maximum LTV - 72.7 %). `Repay` and `Borrow` affect the size of the loan and work similarly with respect to LTV.

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

The `LTV_MAX`, `BID_MIN`, `BID_MAX` parameters are calculated based on the following considerations. The platform takes a commission of 5% for any liquidation (`LIQUIDATION_FEE_RATE`). If the deposit is liquidated with the maximum discount, the remaining part of the profit (5%) goes to the common `STARS` pool, increasing the profitability of deposits, i.e.
```
BID_MIN = 1.1 * loan
```
In order to offer attractive terms to liquidators, you need to have a wide range of discount. In this case it ranges from 0 to 20 %. From here the maximum bid (with the minimum discount) is determined
```
BID_MAX = BID_MIN / (DISCOUNT_MAX - DISCOUNT_MIN)
BID_MAX = 1.375 * loan
```
From here it is easy to get the LTV_MAX value
```
LTV_MAX = loan / BID_MAX
LTV_MAX = 0.727
```

Thus, at the minimum discount, the collateral is redeemed for 110 % of the loan value and 5 % of the profit goes to the `STARS` pool, and at the minimum discount, the collateral is redeemed for 137.5 % of the loan value and 32.5 % of the profit goes to the `STARS` pool. The discount itself varies from 0 to 20 %.
