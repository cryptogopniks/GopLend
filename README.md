### Project description

GopLend is a lending platform that uses NFTs as collateral. The application provides the following features:

1) Depositing STARS on a floating rate deposit with a 2 week unbonding period. This can be a good alternative to STARS staking - with a fixed borrowing rate of 25 % and a platform fee of 5 %, depositors can expect a 20 % APR with an equal liquidity ratio of depositors to borrowers, which exceeds the APR of staking (15.6 % at the time of writing).
Upon opening a deposit, users receive bglSTARS (bonded GopLend STARS) CW20 token in return, which reflect their share of ownership of a common STARS pool on the platform. Over time, this pool will increase due to borrower payments, while the number of bglSTARS will remain unchanged, allowing the latter to grow in value relative to STARS, accumulating the platform's revenue.

2) Arbitrage of bglSTARS. If there are liquidity pools on DEX, some users will be able to exchange bglSTARS for STARS, avoiding the anbonding period, and others will be able to buy bglSTARS with a small discount. The latter option allows not only a more profitable way to become a depositor, but also opens up opportunities for earning on arbitrage, when users can earn by buying bglSTARS on DEX and converting it to STARS on the platform during the rebonding period.

3) STARS loan secured by NFT. Borrowers have clear and convenient conditions - an open-ended loan in STARS with a fixed rate (25 %), you can borrow up to 65.7 % of the pledge amount, liquidation occurs when LTV (loan to value ratio) increases to 73 %. The value of the collateral is calculated taking into account the price of offers, volume of offers, trading volume, which allows to effectively smooth price fluctuations and protect borrowers from liquidation due to market volatility.

4) Collateral Liquidation Bids. Users have a great opportunity to purchase NFTs at a discount by placing bids in STARS at a certain discount to liquidate any NFT, selected collections or specific NFTs. Bids with a smaller discount, narrower coverage and earlier creation date will be prioritized for liquidation.

The application consists of two contracts: `lending-platform` and `oracle`. The first one contains main logic while the second one is required to provide on-chain NFT collections price data. In addition an off-chain scheduler is involved to write data to the `oracle` contract, and an on-chain scheduler to trigger collateral liquidation logic in the `lending-platform` contract.

Any NFT collection must be registered (listed), for which there is a mechanism for dealing with proposals. The process is as follows:
1. The collection creator passes the collection details to the CryptoGopniks team
2. The team verifies the data, decides on the listing, and sets the listing price
3. The admin or the owner of the contract (saved in the contract config by the admin) creates a proposal to add a new collection. The proposal becomes active
4. The creator of the collection (specified in the proposal as collection owner) can accept the proposal by signing a transaction with the required amount. In this case, the status of the proposal will change to "accepted" and the deposited amount will be sent to a special balance, from where the admin/owner can distribute it to the required list of addresses with the specified weights. If necessary, admin/owner can reject the active collection (status will change to "rejected") and create a new one with changed parameters. Also an erroneously registered collection can be deleted by admin/owner

### Deposits

When a user makes a deposit to STARS, bglSTARS is minted to his address in the amount of
```
bglstars_amount = stars_amount * bglstars_supply / stars_pool_volume
```

The price of bglSTARS on the platform is defined as
```
bglstars_price = stars_price * stars_pool_volume / bglstars_supply
```

The price of bglSTARS in pools on DEX will also follow this formula, adjusted by arbitrage.

Initially the price of bglSTARS is equal to the price of STARS, but as the pool grows, it will increase. For example, a user deposited 1000 STARS and received 1000 bglSTARS in return. After some time the pool grew by 20 %. The user made Unbond and burned 500 bglSTARS on the platform. In 2 weeks he will be able to collect 600 STARS.

The yield on deposit is calculated as
```
deposit_apr = (borrow_apr - borrow_commission_rate) * total_borrowed_amount / total_deposited_amount
```

`total_borrowed_amount` may exceed `total_deposited_amount`, as the STARS pool is replenished from borrower payments and income from sales of collateral assets.

Deposit instantly decreases `deposit_apr`. Unbond instantly increases `deposit_apr` without withdrawing liquidity from the STARS pool. Withdraw does not affect `deposit_apr`, but withdraws liquidity from the STARS pool.

### Loans

The borrower can add collateral using DepositCollateral by specifying the desired collections and their tokens. This action reduces LTV. The opposite action is WithdrawCollateral, it cannot be performed if it will increase current LTV to >= 65.7 % (90 % of the maximum LTV - 73 %). Repay and Borrow affect the size of the loan and work similarly with respect to LTV.

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


TODO: add liquidations description
