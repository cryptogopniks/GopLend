import { PriceItem } from "../../common/codegen/Oracle.types";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  Addr,
  ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
  BiddedCollateralItem,
  Collateral,
  CommonConfig,
  CurrencyForToken,
  LiquidationBid,
  QueryBorrowersResponseItem,
  QueryCollateralsResponseItem,
  QueryLiquidationBidsByCollectionAddressListResponseItem,
  RateConfig,
} from "../../common/codegen/LendingPlatform.types";

export const YEAR_IN_SECONDS = 31536000;

export function calcBidMinMultiplier(bidMinRate: number): number {
  return 1 + bidMinRate;
}

export function calcBidMaxMultiplier(
  bidMinRate: number,
  discountMaxRate: number,
  discountMinRate: number
): number {
  return (
    (calcBidMinMultiplier(bidMinRate) * (1 - discountMinRate)) /
    (1 - discountMaxRate)
  );
}

export function calcLtvMax(
  bidMinRate: number,
  discountMaxRate: number,
  discountMinRate: number
): number {
  return 1 / calcBidMaxMultiplier(bidMinRate, discountMaxRate, discountMinRate);
}

export function calcConditionalLtv(
  borrowApr: number,
  amountToBorrow: number,
  accumulatedLoan: number,
  loan: number,
  amountToDeposit: number,
  collateral: number,
  blockTime: number,
  loanCreationDate: number
): number {
  collateral += amountToDeposit;
  accumulatedLoan += amountToBorrow;

  if (!collateral) {
    return 0;
  }

  const borrowDuration = blockTime - loanCreationDate;
  loan =
    accumulatedLoan +
    loan * (1 + (borrowApr * borrowDuration) / YEAR_IN_SECONDS);

  return loan / collateral;
}

export function calcDiscountedAmount(amount: number, discount: number): number {
  return Math.floor(amount * (1 - discount));
}

export function choosePrice(
  mainCurrency: CurrencyForToken,
  priceItem: PriceItem
) {
  return JSON.stringify(mainCurrency.token) ===
    JSON.stringify(priceItem.price.currency.token)
    ? priceItem.price.amount
    : priceItem.usd_price.amount;
}

// Places bid on each collateral token, best bids have higher priority
//
// Bids always liquidate the collateral for from 80 % to 100 % of its value (liquidation price
// is floating), offfers from 100 % or more (liquidation price is fixed). Here are examples:
// 1) Bid for 200 at a 5 % discount. At liquidation, the token price is 100,
// but the liquidation price is 95. After liquidation the bid is 105, you can withdraw it
// and get the balance back
// 2) Offer for 200 with a total discount of 5 %. At liquidation the token price is 100,
// but the liquidation price is 190. After liquidation the offer will be consumed completely
export function matchCollateralsAndBids(
  mainCurrency: CurrencyForToken,
  collectionAndCollateralListByBorrower: [Addr, Collateral][],
  collectionAndBidList: [Addr, LiquidationBid][],
  priceList: PriceItem[]
): BiddedCollateralItem[] {
  // set bids priority
  collectionAndBidList.sort(
    ([_collectionA, bidA], [_collectionB, bidB]) =>
      Number(bidA.amount) - Number(bidB.amount)
  );
  collectionAndBidList.sort(
    ([_collectionA, bidA], [_collectionB, bidB]) =>
      Number(bidA.creation_date) - Number(bidB.creation_date)
  );
  collectionAndBidList.sort(
    ([_collectionA, bidA], [_collectionB, bidB]) =>
      Number(bidA.discount) - Number(bidB.discount)
  );
  collectionAndBidList.sort(
    ([_collectionA, bidA], [_collectionB, bidB]) =>
      Number(bidA.bid_type) - Number(bidB.bid_type)
  );

  // add best bid for each collateral token
  let biddedCollateralList: BiddedCollateralItem[] = [];

  for (const [
    collateral_collection,
    collateral,
  ] of collectionAndCollateralListByBorrower) {
    const priceItem = priceList.find(
      (x) => x.address === collateral_collection
    );
    if (!priceItem) continue;

    const price = choosePrice(mainCurrency, priceItem);

    for (const tokenItem of collateral.token_item_list) {
      for (let i = 0; i < collectionAndBidList.length; i++) {
        const [_bid_collection, bid] = collectionAndBidList[i];

        const { bidAmount, discountedPrice, liquidationPrice } =
          bid.bid_type === "liquidation_bid"
            ? (() => {
                // bids
                const bidAmount = Number(bid.amount);
                const discountedPrice = calcDiscountedAmount(
                  Number(price),
                  Number(bid.discount)
                );
                const liquidationPrice = discountedPrice;

                return { bidAmount, discountedPrice, liquidationPrice };
              })()
            : (() => {
                // offers
                const bidAmount = calcDiscountedAmount(
                  Number(bid.amount),
                  Number(bid.discount)
                );
                const discountedPrice = Number(price);
                const liquidationPrice = bidAmount;

                return { bidAmount, discountedPrice, liquidationPrice };
              })();

        if (
          bid.token_id_list.includes(tokenItem.id) &&
          bidAmount >= discountedPrice
        ) {
          const biddedCollateralItem: BiddedCollateralItem = {
            collection: collateral_collection,
            token_item: tokenItem,
            owner: collateral.owner,
            liquidator: bid.liquidator,
            bid_creation_date: bid.creation_date,
            liquidation_price: liquidationPrice.toString(),
            collateral_price: price,
            bid_amount: bid.amount,
            bid_discount: bid.discount,
            bid_type: bid.bid_type,
          };

          biddedCollateralList.push(biddedCollateralItem);

          collectionAndBidList[i][1].amount = (
            bidAmount - discountedPrice
          ).toString();
          break;
        }
      }
    }
  }

  return biddedCollateralList;
}

// ltvMax = (loan - (1 - discount) * liquidation_value) / (collateral - liquidation_value)
// ltvMax * collateral - ltvMax * liquidation_value = loan - (1 - discount) * liquidation_value
// (1 - discount - ltvMax) * liquidation_value = loan - ltvMax * collateral
// liquidation_value = (loan - ltvMax * collateral) / (1 - discount - ltvMax)
// liquidation_value = collateral * (ltv - ltvMax) / (1 - discount - ltvMax)
export function calcLiquidationValue(
  collateral: number,
  discountRate: number,
  ltv: number,
  ltvMax: number
): number {
  return Math.min(
    (collateral * (ltv - ltvMax)) / (1 - discountRate - ltvMax),
    collateral
  );
}

export function calcLiquidationLtv(
  biddedCollateral: BiddedCollateralItem[],
  borrowApr: number,
  accumulatedLoan: number,
  loan: number,
  collateral: number,
  blockTime: number,
  loanCreationDate: number
): number {
  const loanDiff = biddedCollateral.reduce(
    (acc, cur) => acc + Number(cur.liquidation_price),
    0
  );
  const collateralDiff = biddedCollateral.reduce(
    (acc, cur) => acc + Number(cur.collateral_price),
    0
  );

  collateral -= collateralDiff;

  if (!collateral) {
    return 0;
  }

  const borrowDuration = blockTime - loanCreationDate;
  loan =
    accumulatedLoan +
    loan * (1 + (borrowApr * borrowDuration) / YEAR_IN_SECONDS) -
    loanDiff;

  return loan / collateral;
}

// there is no way to calculate liquidation_value properly
// then ltv must be calculated on each iteration
export function calcLiquidationSet(
  biddedCollateralList: BiddedCollateralItem[],
  maxLiquidationValue: number,
  ltvMax: number,
  borrowApr: number,
  accumulatedLoan: number,
  loan: number,
  collateral: number,
  blockTime: number,
  loanCreationDate: number
): BiddedCollateralItem[] {
  const UPPER_VALUE_MULTIPLIER = 1.5;

  // sort ascending by creation_date
  biddedCollateralList.sort(
    (a, b) => Number(a.bid_creation_date) - Number(b.bid_creation_date)
  );
  // sort ascending by discount
  biddedCollateralList.sort(
    (a, b) => Number(a.bid_discount) - Number(b.bid_discount)
  );
  // sort descending by liquidation_price
  biddedCollateralList.sort(
    (a, b) => Number(b.liquidation_price) - Number(a.liquidation_price)
  );

  // get subvector with values <= liquidation_value
  // [19, 16, 14, 13, 10, 7, 5, 5, 2] -> [10, 7, 5, 5, 2]
  const subvector: BiddedCollateralItem[] = biddedCollateralList.filter(
    (x) => Number(x.liquidation_price) <= maxLiquidationValue
  );

  // get smallest collateral_list item > liquidation_value or 1.5 * liquidation_value if it isn't found
  const upper_value =
    Number(
      biddedCollateralList
        .reverse()
        .find((x) => Number(x.liquidation_price) > maxLiquidationValue)
        ?.liquidation_price
    ) || maxLiquidationValue * UPPER_VALUE_MULTIPLIER;

  // iterate over subvector, compare with [liquidation_value, upper_value] and generate vectors
  let result: BiddedCollateralItem[][] = [];

  for (let i = 0; i < subvector.length; i++) {
    const bidded_collateral_item = subvector[i];
    let temp: BiddedCollateralItem[] = [bidded_collateral_item];
    let collateral_sum: number = Number(
      bidded_collateral_item.liquidation_price
    );

    let ltv = calcLiquidationLtv(
      temp,
      borrowApr,
      accumulatedLoan,
      loan,
      collateral,
      blockTime,
      loanCreationDate
    );

    if (ltv <= ltvMax) {
      result.push(temp);
      continue;
    }

    for (const bidded_collateral_item_next of subvector.slice(
      i + 1,
      subvector.length
    )) {
      let collateral_sum_next =
        collateral_sum + Number(bidded_collateral_item_next.liquidation_price);

      if (collateral_sum_next >= upper_value) {
        continue;
      }

      temp.push(bidded_collateral_item_next);

      let ltv = calcLiquidationLtv(
        temp,
        borrowApr,
        accumulatedLoan,
        loan,
        collateral,
        blockTime,
        loanCreationDate
      );

      if (ltv <= ltvMax) {
        result.push(temp);
        temp.pop();
        continue;
      }

      collateral_sum = collateral_sum_next;
    }
  }

  // liquidate first more expensive collateral items
  result.sort((a, b) => a.length - b.length);
  // prefer sets with lower sum discount
  result.sort(
    (a, b) =>
      a.reduce((acc, cur) => acc + Number(cur.bid_discount), 0) -
      b.reduce((acc, cur) => acc + Number(cur.bid_discount), 0)
  );
  // choose cheapest set
  result.sort(
    (a, b) =>
      a.reduce((acc, cur) => acc + Number(cur.liquidation_price), 0) -
      b.reduce((acc, cur) => acc + Number(cur.liquidation_price), 0)
  );

  return result[1] || biddedCollateralList;
}

export function getCollateralsByBorrower(
  collateralList: QueryCollateralsResponseItem[],
  borrowerAddress: Addr
): [Addr, Collateral][] {
  return collateralList.reduce((acc, { address: collection, collateral }) => {
    const ownedCollateral = collateral.find(
      ({ owner }) => owner === borrowerAddress
    );

    if (ownedCollateral) {
      acc.push([collection, ownedCollateral]);
    }

    return acc;
  }, [] as [Addr, Collateral][]);
}

export function getCollateralValue(
  mainCurrency: CurrencyForToken,
  collectionAndCollateralListByBorrower: [Addr, Collateral][],
  prices: PriceItem[]
): number | undefined {
  let isPriceOutdated = false;

  const collateralValue = collectionAndCollateralListByBorrower.reduce(
    (acc, [collectionAddress, collateral]) => {
      const priceItem = prices.find((x) => x.address === collectionAddress);
      if (!priceItem) return acc;

      if (priceItem.is_outdated) {
        isPriceOutdated = true;
      }

      const price = choosePrice(mainCurrency, priceItem);

      return acc + Number(price) * collateral.token_item_list.length;
    },
    0
  );

  return !isPriceOutdated ? collateralValue : undefined;
}

interface TargetBorrower {
  platformType: PlatformType;
  borrowerAddress: string;
  ltv: number;
  loanAmount: number;
  loanCreationDate: number;
  accumulatedLoan: number;
  collateralValue: number;
  collectionAndCollateralList: [string, Collateral][];
}

// get borrowers with ltv > ltvMax
export function getTargetBorrowers(
  platformType: PlatformType,
  borrowerList: QueryBorrowersResponseItem[],
  collateralList: QueryCollateralsResponseItem[],
  priceList: PriceItem[],
  rateConfig: RateConfig,
  commonConfig: CommonConfig,
  ltvMax: number,
  blockTime: number
): TargetBorrower[] {
  return borrowerList.reduce(
    (
      acc,
      { address: borrowerAddress, borrower: { loan, accumulated_loan } }
    ) => {
      const collectionAndCollateralListByBorrower = getCollateralsByBorrower(
        collateralList,
        borrowerAddress
      );

      const collateralValue =
        getCollateralValue(
          commonConfig.main_currency,
          collectionAndCollateralListByBorrower,
          priceList
        ) || 0;

      const loanAmount = Number(loan.amount);
      const accumulatedLoan = Number(accumulated_loan);
      const ltv = calcConditionalLtv(
        Number(rateConfig.borrow_apr),
        0,
        accumulatedLoan,
        loanAmount,
        0,
        collateralValue,
        blockTime,
        loan.creation_date
      );

      if (ltv > ltvMax) {
        acc.push({
          platformType,
          borrowerAddress,
          ltv,
          loanAmount,
          loanCreationDate: loan.creation_date,
          accumulatedLoan,
          collateralValue,
          collectionAndCollateralList: collectionAndCollateralListByBorrower,
        });
      }

      return acc;
    },
    [] as TargetBorrower[]
  );
}

// find bids containing at least 1 borrower's collateral token
export function getCollectionAndBidList(
  liquidationBids: ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
  collectionAndCollateralListByBorrower: [Addr, Collateral][]
): [Addr, LiquidationBid][] {
  let collectionAndBidList: [Addr, LiquidationBid][] = [];

  for (const { collection_address, liquidation_bids } of liquidationBids) {
    for (const bid of liquidation_bids) {
      const matchingCollateral = collectionAndCollateralListByBorrower.some(
        ([collection, collateral]) =>
          collection === collection_address &&
          bid.token_id_list.some((bidTokenId) =>
            collateral.token_item_list.some(({ id }) => id === bidTokenId)
          )
      );

      if (matchingCollateral) {
        collectionAndBidList.push([collection_address, bid]);
      }
    }
  }

  return collectionAndBidList;
}

// TODO: collection offer must be consumed completely from both offer lists
export function reduceBids(
  totalBids: QueryLiquidationBidsByCollectionAddressListResponseItem[],
  liquidationSet: BiddedCollateralItem[]
): QueryLiquidationBidsByCollectionAddressListResponseItem[] {
  return totalBids.reduce((acc, { collection_address, liquidation_bids }) => {
    const liquidationSetByCollection = liquidationSet.filter(
      (x) => x.collection === collection_address
    );

    if (!liquidationSetByCollection.length) {
      acc.push({ collection_address, liquidation_bids });
      return acc;
    }

    const liquidationBids = liquidation_bids.reduce((acc, cur) => {
      let bidAmount = Number(cur.amount);
      let tokenIdList = cur.token_id_list;

      liquidationSetByCollection.forEach(
        ({ token_item, liquidation_price }) => {
          if (tokenIdList.includes(token_item.id)) {
            bidAmount -= Math.min(Number(liquidation_price), bidAmount);
            tokenIdList = tokenIdList.filter((id) => id !== token_item.id);
          }
        }
      );

      if (tokenIdList.length && !bidAmount) {
        acc.push({
          ...cur,
          amount: bidAmount.toString(),
          token_id_list: tokenIdList,
        });
      }

      return acc;
    }, [] as LiquidationBid[]);

    if (!liquidationBids.length) {
      return acc;
    }

    acc.push({ collection_address, liquidation_bids: liquidationBids });
    return acc;
  }, [] as QueryLiquidationBidsByCollectionAddressListResponseItem[]);
}

export function getCollectionsFromLtvAndCollateralList(
  ltvAndCollateralList: TargetBorrower[]
): string[] {
  return ltvAndCollateralList.reduce((acc, { collectionAndCollateralList }) => {
    collectionAndCollateralList.forEach(([collectionAddress]) => {
      if (!acc.includes(collectionAddress)) {
        acc.push(collectionAddress);
      }
    });

    return acc;
  }, [] as string[]);
}
