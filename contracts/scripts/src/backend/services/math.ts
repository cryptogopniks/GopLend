import {
  Addr,
  BiddedCollateralItem,
  Collateral,
  LiquidationBid,
} from "../../common/codegen/LendingPlatform.types";
import { PriceItem } from "../../common/codegen/Oracle.types";

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

export function calcDiscountedPrice(price: number, discount: number): number {
  return price * (1 - discount);
}

// places bid on each collateral token, best bids have higher priority
export function matchCollateralsAndBids(
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
      Number(bidA.is_offer) - Number(bidB.is_offer)
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

    const { price } = priceItem;

    for (const token_id of collateral.token_id_list) {
      for (let i = 0; i < collectionAndBidList.length; i++) {
        const [_bid_collection, bid] = collectionAndBidList[i];

        let discounted_price = calcDiscountedPrice(
          Number(price.amount),
          Number(bid.discount)
        );

        if (
          bid.token_id_list.includes(token_id) &&
          Number(bid.amount) >= discounted_price
        ) {
          const biddedCollateralItem: BiddedCollateralItem = {
            collection: collateral_collection,
            token_id: token_id,
            owner: collateral.owner,
            liquidator: bid.liquidator,
            bid_creation_date: bid.creation_date,
            liquidation_price: discounted_price.toString(),
            collateral_price: price.amount,
            bid_discount: bid.discount,
            is_offer: bid.is_offer,
          };

          biddedCollateralList.push(biddedCollateralItem);

          collectionAndBidList[i][1].amount = (
            Number(bid.amount) - discounted_price
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
