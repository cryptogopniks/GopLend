import { floor, getLast } from "../../common/utils";
import { PriceItemForFundsForToken } from "../../common/codegen/Oracle.types";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  LiquidationBidExtended,
  TargetBorrower,
} from "../../common/interfaces";
import {
  Bid,
  CollectionBid,
} from "../../common/interfaces/stargaze-marketplace";
import {
  Addr,
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
  ltvMax: number,
  borrowApr: number,
  amountToBorrow: number,
  amountToRepay: number,
  accumulatedLoan: number,
  loan: number,
  amountToDeposit: number,
  amountToWithdraw: number,
  collateral: number,
  blockTime: number,
  loanCreationDate: number
): number {
  const borrowDuration = blockTime - loanCreationDate;
  collateral += amountToDeposit - amountToWithdraw;
  accumulatedLoan +=
    loan * (1 + (borrowApr * borrowDuration) / YEAR_IN_SECONDS);

  loan = accumulatedLoan + amountToBorrow - amountToRepay;

  // if loan is zero then ltv is zero anyway
  if (!loan) {
    return 0;
  }

  // if loan isn't zero while collateral is zero then ltv is max
  if (!collateral) {
    return ltvMax;
  }

  return loan / collateral;
}

export function calcDiscountedAmount(amount: number, discount: number): number {
  return Math.floor(amount * (1 - discount));
}

export function choosePrice(
  mainCurrency: CurrencyForToken,
  priceItem: PriceItemForFundsForToken
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
export function matchCollateralsAndBidsLegacy(
  mainCurrency: CurrencyForToken,
  collectionAndCollateralListByBorrower: [Addr, Collateral][],
  collectionAndBidList: [Addr, LiquidationBid][],
  priceList: PriceItemForFundsForToken[]
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
  collectionAndBidList: LiquidationBidExtended[],
  priceList: PriceItemForFundsForToken[]
): BiddedCollateralItem[] {
  // set bids priority
  collectionAndBidList.sort(
    (bidA, bidB) => Number(bidA.amount) - Number(bidB.amount)
  );
  collectionAndBidList.sort(
    (bidA, bidB) => Number(bidA.creationDate) - Number(bidB.creationDate)
  );
  collectionAndBidList.sort(
    (bidA, bidB) => Number(bidA.discount) - Number(bidB.discount)
  );
  collectionAndBidList.sort(
    (bidA, bidB) => Number(bidA.bidType) - Number(bidB.bidType)
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
        const bid = collectionAndBidList[i];

        const { bidAmount, discountedPrice, liquidationPrice } =
          bid.bidType === "liquidation_bid"
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
          bid.tokenIdList.includes(tokenItem.id) &&
          bidAmount >= discountedPrice
        ) {
          const biddedCollateralItem: BiddedCollateralItem = {
            collection: collateral_collection,
            token_item: tokenItem,
            owner: collateral.owner,
            liquidator: bid.liquidator,
            bid_creation_date: bid.creationDate,
            liquidation_price: liquidationPrice.toString(),
            collateral_price: price,
            bid_amount: bid.amount.toString(),
            bid_discount: bid.discount.toString(),
            bid_type: bid.bidType,
          };

          biddedCollateralList.push(biddedCollateralItem);

          collectionAndBidList[i].amount = bidAmount - discountedPrice;
          break;
        }
      }
    }
  }

  return biddedCollateralList;
}

// targetLtv = (loan - (1 - discount) * liquidation_value) / (collateral - liquidation_value)
// targetLtv * collateral - targetLtv * liquidation_value = loan - (1 - discount) * liquidation_value
// (1 - discount - targetLtv) * liquidation_value = loan - targetLtv * collateral
// liquidation_value = (loan - targetLtv * collateral) / (1 - discount - targetLtv)
// liquidation_value = collateral * (ltv - targetLtv) / (1 - discount - targetLtv)
// targetLtv = ltvMaxFraction * ltvMax
export function calcLiquidationValue(
  collateral: number,
  discountRate: number,
  ltv: number,
  ltvMax: number,
  ltvMaxFraction: number
): number {
  if (ltvMaxFraction < 0.5 || ltvMaxFraction > 1) {
    throw new Error("ltvMaxFraction is out of range [0.5, 1]!");
  }

  const targetLtv = ltvMaxFraction * ltvMax;

  return Math.min(
    (collateral * (ltv - targetLtv)) / (1 - discountRate - targetLtv),
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
  prices: PriceItemForFundsForToken[]
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

// get borrowers with ltv > ltvMax
// no collateral borrowers will be ignored
export function getTargetBorrowers(
  platformType: PlatformType,
  borrowerList: QueryBorrowersResponseItem[],
  collateralList: QueryCollateralsResponseItem[],
  priceList: PriceItemForFundsForToken[],
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
        ltvMax,
        Number(rateConfig.borrow_apr),
        0,
        0,
        accumulatedLoan,
        loanAmount,
        0,
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
  liquidationBids: LiquidationBidExtended[],
  collectionAndCollateralListByBorrower: [Addr, Collateral][]
): LiquidationBidExtended[] {
  let collectionAndBidList: LiquidationBidExtended[] = [];

  for (const bid of liquidationBids) {
    const matchingCollateral = collectionAndCollateralListByBorrower.some(
      ([collection, collateral]) =>
        collection === bid.collectionAddress &&
        bid.tokenIdList.some((bidTokenId) =>
          collateral.token_item_list.some(({ id }) => id === bidTokenId)
        )
    );

    if (matchingCollateral) {
      collectionAndBidList.push(bid);
    }
  }

  return collectionAndBidList;
}

export function reduceBids(
  platformType: PlatformType,
  totalBidsMain: LiquidationBidExtended[],
  totalBidsStable: LiquidationBidExtended[],
  liquidationSet: BiddedCollateralItem[]
): {
  totalBidsMain: LiquidationBidExtended[];
  totalBidsStable: LiquidationBidExtended[];
} {
  let [totalBidsPrimary, totalBidsSecondary] =
    platformType === "main"
      ? [totalBidsMain, totalBidsStable]
      : [totalBidsStable, totalBidsMain];

  for (const { collection, token_item, liquidation_price } of liquidationSet) {
    totalBidsPrimary = totalBidsPrimary.reduce((acc, bid) => {
      if (
        bid.collectionAddress !== collection ||
        !bid.tokenIdList.includes(token_item.id)
      ) {
        acc.push(bid);
        return acc;
      }

      if (bid.bidType === "collection_offer") {
        totalBidsSecondary = totalBidsSecondary.filter(
          (x) => x.collectionOfferId !== bid.collectionOfferId
        );
        return acc;
      }

      bid.amount = floor(
        bid.amount - Math.min(Number(liquidation_price), bid.amount)
      );
      bid.tokenIdList = bid.tokenIdList.filter((id) => id !== token_item.id);
      if (bid.tokenIdList.length && !bid.amount) {
        acc.push(bid);
      }
      return acc;
    }, [] as LiquidationBidExtended[]);
  }

  return { totalBidsMain, totalBidsStable };
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

// imagine we have 500 STARS offer, 1 STARS is 0.01 USDC
// if we will assume the offer is 5 USDC we will lose some funds due to
// swap fee, price impact, price ratio fluctuation
// expecting 2 % losses we can accept 4.9 USDC offer
//
// price = base_price * price_ratio * (1 - price_impact)
// as price_impact comes with price_ratio
// price = base_price * price_ratio
export function calcPrice(basePrice: number, priceRatio: number): number {
  return basePrice * priceRatio;
}

export function distributeOffers(
  collectionList: string[],
  collateralListMain: QueryCollateralsResponseItem[],
  collateralListStable: QueryCollateralsResponseItem[],
  tokenOfferList: [string, Bid[]][],
  collectionOfferList: [string, CollectionBid[]][],
  discountBaseList: [string, number][],
  priceRatioStable: number,
  blockTime: number,
  platformList: PlatformType[]
): LiquidationBidExtended[] {
  let offers: LiquidationBidExtended[] = [];
  let collectionOfferId = 0;

  for (const collection of collectionList) {
    const tokenOffers =
      tokenOfferList.find(([address]) => address === collection)?.[1] || [];
    const collectionOffers =
      collectionOfferList.find(([address]) => address === collection)?.[1] ||
      [];
    if (!tokenOffers.length && !collectionOffers.length) continue;

    const discountBase =
      discountBaseList.find(([address]) => address === collection)?.[1] || 0;

    const collateralMain =
      collateralListMain.find((x) => x.address === collection)?.collateral ||
      [];
    const collateralStable =
      collateralListStable.find((x) => x.address === collection)?.collateral ||
      [];

    const tokenListMain = collateralMain.flatMap((x) =>
      x.token_item_list.map((y) => y.id)
    );
    const tokenListStable = collateralStable.flatMap((x) =>
      x.token_item_list.map((y) => y.id)
    );
    if (!tokenListMain.length && !tokenListStable.length) continue;

    for (const { price, bidder, token_id, finders_fee_bps } of tokenOffers) {
      const findersFeeRate = Number(finders_fee_bps) / 100;
      const tokenId = token_id.toString();

      const isMain = collateralMain.find((x) =>
        x.token_item_list.some((y) => y.id === tokenId)
      );
      const isStable = collateralStable.find((x) =>
        x.token_item_list.some((y) => y.id === tokenId)
      );
      if (!isMain && !isStable) continue;

      offers.push({
        creationDate: blockTime,
        liquidator: bidder,
        tokenIdList: [tokenId],
        amount: calcPrice(Number(price), isMain ? 1 : priceRatioStable),
        discount: discountBase + findersFeeRate,
        bidType: "token_offer",
        platformType: isMain ? "main" : "stable",
        collectionOfferId: -1,
        collectionAddress: collection,
      });
    }

    for (const { price, bidder, finders_fee_bps } of collectionOffers) {
      const findersFeeRate = Number(finders_fee_bps) / 100;

      for (const platformType of platformList) {
        const isMain = platformType === "main";
        const tokenIdList = isMain ? tokenListMain : tokenListStable;
        if (!tokenIdList.length) continue;

        offers.push({
          creationDate: blockTime,
          liquidator: bidder,
          tokenIdList,
          amount: calcPrice(Number(price), isMain ? 1 : priceRatioStable),
          discount: discountBase + findersFeeRate,
          bidType: "collection_offer",
          platformType,
          collectionOfferId,
          collectionAddress: collection,
        });
      }

      collectionOfferId++;
    }
  }

  return offers;
}

export function mergeBidsAndOffers(
  liquidationBidsMain: QueryLiquidationBidsByCollectionAddressListResponseItem[],
  liquidationBidsStable: QueryLiquidationBidsByCollectionAddressListResponseItem[],
  offers: LiquidationBidExtended[]
): {
  totalBidsMain: LiquidationBidExtended[];
  totalBidsStable: LiquidationBidExtended[];
} {
  const [totalBidsMain, totalBidsStable] = offers.reduce(
    (acc, cur) => {
      acc[cur.platformType === "main" ? 0 : 1].push(cur);
      return acc;
    },
    [[], []] as [LiquidationBidExtended[], LiquidationBidExtended[]]
  );

  const liquidationBidsMainExtended: LiquidationBidExtended[] =
    liquidationBidsMain.flatMap(({ collection_address, liquidation_bids }) => {
      return liquidation_bids.map((x) => ({
        creationDate: x.creation_date,
        liquidator: x.liquidator,
        tokenIdList: x.token_id_list,
        amount: Number(x.amount),
        discount: Number(x.discount),
        bidType: x.bid_type,
        platformType: "main",
        collectionOfferId: -1,
        collectionAddress: collection_address,
      }));
    });

  const liquidationBidsStableExtended: LiquidationBidExtended[] =
    liquidationBidsStable.flatMap(
      ({ collection_address, liquidation_bids }) => {
        return liquidation_bids.map((x) => ({
          creationDate: x.creation_date,
          liquidator: x.liquidator,
          tokenIdList: x.token_id_list,
          amount: Number(x.amount),
          discount: Number(x.discount),
          bidType: x.bid_type,
          platformType: "stable",
          collectionOfferId: -1,
          collectionAddress: collection_address,
        }));
      }
    );

  return {
    totalBidsMain: [...liquidationBidsMainExtended, ...totalBidsMain],
    totalBidsStable: [...liquidationBidsStableExtended, ...totalBidsStable],
  };
}

// returns [collection, tokenList][]
export function aggregateTokensByCollection(
  ltvAndCollateralList: TargetBorrower[],
  collectionList: string[]
): [string, string[]][] {
  return collectionList.map((collectionAddress) => {
    const tokens = ltvAndCollateralList.reduce(
      (acc, { collectionAndCollateralList }) => {
        const currentTokens =
          collectionAndCollateralList
            .find(([collection]) => collection === collectionAddress)?.[1]
            .token_item_list.map(({ id }) => id) || [];

        return [...acc, ...currentTokens];
      },
      [] as string[]
    );

    return [collectionAddress, tokens];
  });
}

export interface PriceSample {
  value: number;
  timestamp: number;
}

function applyWindow(
  window: number,
  priceSampleList: PriceSample[],
  newPriceSample?: PriceSample
): PriceSample[] {
  const { timestamp: lastTimestamp } =
    getLast(priceSampleList) || newPriceSample;
  const lowerBoundary = lastTimestamp - Math.min(window, lastTimestamp);

  let [boundarySample] = priceSampleList;
  let updatedPriceSampleList = priceSampleList.reduce(
    (acc, { value, timestamp }) => {
      if (timestamp > lowerBoundary) {
        acc.push({ value, timestamp });
      } else {
        boundarySample = { value, timestamp: lowerBoundary };
      }

      return acc;
    },
    [] as PriceSample[]
  );

  const [firstSample] = updatedPriceSampleList;
  if (
    boundarySample?.timestamp !== firstSample?.timestamp &&
    boundarySample?.value !== firstSample?.value
  ) {
    updatedPriceSampleList = [boundarySample, ...updatedPriceSampleList];
  }

  return updatedPriceSampleList;
}

// returns [twap, updatedPriceSampleList]
export function calcTwap(
  window: number,
  priceSampleList: PriceSample[],
  newPriceSample: PriceSample
): [number, PriceSample[]] {
  if (newPriceSample) {
    priceSampleList = [...priceSampleList, newPriceSample];
  }

  if (!priceSampleList.length) {
    return [0, priceSampleList];
  }

  const updatedPriceSampleList = applyWindow(window, priceSampleList);
  const [firstPriceSample] = updatedPriceSampleList;
  if (updatedPriceSampleList.length === 1) {
    return [firstPriceSample.value, updatedPriceSampleList];
  }

  const [_, period, product] = updatedPriceSampleList.reduce(
    ([prevSample, periodAcc, productAcc], { value, timestamp }) => {
      const period: number = timestamp - prevSample.timestamp;
      return [
        { value, timestamp },
        periodAcc + period,
        productAcc + period * prevSample.value,
      ];
    },
    [firstPriceSample, 0, 0]
  );
  const twap = floor(period ? product / period : 0, 6);

  return [twap, updatedPriceSampleList];
}

export function calcEma(
  windowSamples: number,
  newPrice: number,
  prevEma?: number
): number {
  if (!prevEma) {
    return newPrice;
  }

  const k = 2 / (windowSamples + 1);
  return floor(newPrice * k + prevEma * (1 - k), 6);
}

// combine TWAP and EMA
export function calcHybridAverage(
  twap: number,
  ema: number,
  twapWeight: number
): number {
  return twapWeight * twap + (1 - twapWeight) * ema;
}

export function calcMean(values: number[]): number {
  return values.reduce((acc, cur) => acc + cur, 0) / values.length;
}

export interface CollectionPriceResponse {
  twapSampleList: PriceSample[];
  floorPriceEma: number;
  floorPrice: number; // for market maker script
  twapOfferList: PriceSample[];
  offerEma: number;
  collectionPrice: number;
}

export function calcCollectionPrice(
  floorPriceBoundaryUpper: number,
  floorPriceBoundaryLower: number,
  twapWeight: number,
  window: number,
  fpTwapWindowMultiplier: number,
  floorPriceSampleList: PriceSample[],
  newFloorPriceSample: PriceSample,
  prevFloorPriceEma: number | undefined,
  offerPriceList: number[],
  feeRateList: number[],
  meanOfferSampleList: PriceSample[],
  prevMeanOfferEma: number
): CollectionPriceResponse {
  // hybrid TWAP/EMA floor price
  // FP must be smoothed stronger as FP is more volatile than mean of offers
  const [floorPriceTwap, twapSampleList] = calcTwap(
    fpTwapWindowMultiplier * window,
    floorPriceSampleList,
    newFloorPriceSample
  );

  const [firstPriceSample] = floorPriceSampleList;
  const lastPriceSample = newFloorPriceSample;
  const fullPeriod = firstPriceSample
    ? lastPriceSample.timestamp - firstPriceSample.timestamp
    : 0;
  const averageSamplePeriod = fullPeriod / (floorPriceSampleList.length + 1);
  const windowSamples = averageSamplePeriod ? window / averageSamplePeriod : 0;

  const floorPriceEma = calcEma(
    windowSamples,
    newFloorPriceSample.value,
    prevFloorPriceEma
  );
  const floorPrice = calcHybridAverage(
    floorPriceTwap,
    floorPriceEma,
    twapWeight
  );

  // filter offers
  const validOffers = offerPriceList.reduce((acc, offer, i) => {
    if (
      offer >= floorPriceBoundaryLower * floorPrice &&
      offer <= floorPriceBoundaryUpper * floorPrice
    ) {
      acc.push(floor((1 - feeRateList[i]) * offer));
    }

    return acc;
  }, [] as number[]);

  // mean of filtered offers
  const meanOffer = validOffers.length
    ? calcMean(validOffers)
    : (floorPrice * (floorPriceBoundaryUpper + floorPriceBoundaryLower)) / 2;
  const newMeanOfferSample: PriceSample = {
    value: meanOffer,
    timestamp: newFloorPriceSample?.timestamp,
  };

  // hybrid TWAP/EMA of mean offers
  const [offerTwap, twapOfferList] = calcTwap(
    window,
    meanOfferSampleList,
    newMeanOfferSample
  );
  const offerEma = calcEma(
    windowSamples,
    newMeanOfferSample.value,
    prevMeanOfferEma
  );
  const collectionPrice = calcHybridAverage(offerTwap, offerEma, twapWeight);

  return {
    twapSampleList,
    floorPriceEma,
    floorPrice,
    twapOfferList,
    offerEma,
    collectionPrice,
  };
}

export function splitOffers(
  lowerFloorPrice: number,
  upperFloorPrice: number,
  price_and_outpost_list: [string, string][]
) {
  const priceAndOutpostList = (
    price_and_outpost_list.map(([price, outpost]) => [
      Number(price),
      outpost,
    ]) as [number, string][]
  ).sort(([priceA], [priceB]) => priceB - priceA);

  const regularOffers = priceAndOutpostList
    .map(([price]) => price)
    .filter((price) => price >= lowerFloorPrice && price <= upperFloorPrice);

  // sort ascending as we need to get rid of smallest offers first
  const tooSmallOffers = priceAndOutpostList
    .filter(([price]) => price < lowerFloorPrice)
    .sort(([priceA], [priceB]) => priceA - priceB);

  const tooBigOffers = priceAndOutpostList.filter(
    ([price]) => price > upperFloorPrice
  );

  return {
    regularOffers,
    tooSmallOffers,
    tooBigOffers,
  };
}

export function getOffersExtendedUp(
  upper_allowed_offers_boundary: number,
  amountUndistributed: number,
  floorPrice: number,
  lowerFloorPrice: number,
  regularOffers: number[],
  tooSmallOffers: [number, string][]
) {
  const highestOffer = regularOffers[0] || lowerFloorPrice;
  const allowedPriceRange =
    floorPrice * upper_allowed_offers_boundary - highestOffer;
  const priceStep = floor(allowedPriceRange / (tooSmallOffers.length - 1));
  const fromToPriceAndOutpostListRaw: [number, number, string][] =
    tooSmallOffers.map(([priceBefore, outpost], i) => {
      const priceAfter = highestOffer + priceStep * (i + 1);
      return [priceBefore, priceAfter, outpost];
    });

  // filter offers based on available liquidity
  const [fromToPriceAndOutpostList]: [[number, number, string][], number] =
    fromToPriceAndOutpostListRaw.reduce(
      (acc, cur) => {
        let [list, availableLiquidity] = acc;
        const [priceBefore, priceAfter] = cur;

        if (availableLiquidity >= priceAfter) {
          list = [...list, cur];
          availableLiquidity -= priceAfter - priceBefore;
        }

        return [list, availableLiquidity];
      },
      [[], amountUndistributed] as [[number, number, string][], number]
    );

  const isNoLiquidity = !fromToPriceAndOutpostList.length;
  const isRequiredMoreLiquidity =
    fromToPriceAndOutpostList.length < fromToPriceAndOutpostListRaw.length;

  return {
    fromToPriceAndOutpostList,
    isNoLiquidity,
    isRequiredMoreLiquidity,
  };
}

export function getOffersExtendedDown(
  lower_allowed_offers_boundary: number,
  amountUndistributed: number,
  floorPrice: number,
  upperFloorPrice: number,
  regularOffers: number[],
  tooBigOffers: [number, string][]
) {
  const lowestOffer = getLast(regularOffers) || upperFloorPrice;
  const allowedPriceRange =
    lowestOffer - floorPrice * lower_allowed_offers_boundary;
  const priceStep = floor(allowedPriceRange / (tooBigOffers.length - 1));
  const fromToPriceAndOutpostList: [number, number, string][] =
    tooBigOffers.map(([priceBefore, outpost], i) => {
      const priceAfter = lowestOffer - priceStep * (i + 1);
      return [priceBefore, priceAfter, outpost];
    });

  const [[_, firstPriceAfter]] = fromToPriceAndOutpostList;
  const isNoLiquidity = amountUndistributed < firstPriceAfter;

  return {
    fromToPriceAndOutpostList,
    isNoLiquidity,
  };
}
