import {
  getOffersExtendedDown,
  getOffersExtendedUp,
  splitOffers,
} from "../services/math";

const M: number = 1_000_000;
const MAX_OFFERS = 20;
const FLOOR_PRICE_BOUNDARY = {
  UPPER: 0.93,
  LOWER: 0.8,
};
const ALLOWED_OFFERS_BOUNDARY = {
  UPPER: 0.93,
  LOWER: 0.8,
};

describe("testing splitOffers and getOffersExtendedUp/getOffersExtendedDown", () => {
  test("unchanged price", () => {
    const floorPrice: number = 1_000 * M;
    const priceList: string[] = [
      800 * M,
      806.5 * M,
      813 * M,
      819.5 * M,
      826 * M,
      832.5 * M,
      839 * M,
      845.5 * M,
      852 * M,
      858.5 * M,
      865 * M,
      871.5 * M,
      878 * M,
      884.5 * M,
      891 * M,
      897.5 * M,
      904 * M,
      910.5 * M,
      917 * M,
      923.5 * M,
      930 * M,
    ].map((x) => x.toString());
    const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
    const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
    const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList
    );

    const expectedRegularOffers: number[] = [
      930 * M,
      923.5 * M,
      917 * M,
      910.5 * M,
      904 * M,
      897.5 * M,
      891 * M,
      884.5 * M,
      878 * M,
      871.5 * M,
      865 * M,
      858.5 * M,
      852 * M,
      845.5 * M,
      839 * M,
      832.5 * M,
      826 * M,
      819.5 * M,
      813 * M,
      806.5 * M,
      800 * M,
    ];
    const expectedTooSmallOffers: number[] = [];
    const expectedTooBigOffers: number[] = [];

    expect(regularOffers).toStrictEqual(expectedRegularOffers);
    expect(tooSmallOffers).toStrictEqual(expectedTooSmallOffers);
    expect(tooBigOffers).toStrictEqual(expectedTooBigOffers);
  });

  test("fp was increased", () => {
    const amountUndistributed: number = 10_000 * M;
    const floorPrice: number = 1_040 * M;
    const priceList: string[] = [
      800 * M,
      806.5 * M,
      813 * M,
      819.5 * M,
      826 * M,
      832.5 * M,
      839 * M,
      845.5 * M,
      852 * M,
      858.5 * M,
      865 * M,
      871.5 * M,
      878 * M,
      884.5 * M,
      891 * M,
      897.5 * M,
      904 * M,
      910.5 * M,
      917 * M,
      923.5 * M,
      930 * M,
    ].map((x) => x.toString());
    const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
    const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
    const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList
    );

    const expectedRegularOffers: number[] = [
      930 * M,
      923.5 * M,
      917 * M,
      910.5 * M,
      904 * M,
      897.5 * M,
      891 * M,
      884.5 * M,
      878 * M,
      871.5 * M,
      865 * M,
      858.5 * M,
      852 * M,
      845.5 * M,
      839 * M,
      832.5 * M,
    ];
    const expectedTooSmallOffers: number[] = [
      800 * M,
      806.5 * M,
      813 * M,
      819.5 * M,
      826 * M,
    ];
    const expectedTooBigOffers: number[] = [];

    expect(regularOffers).toStrictEqual(expectedRegularOffers);
    expect(tooSmallOffers).toStrictEqual(expectedTooSmallOffers);
    expect(tooBigOffers).toStrictEqual(expectedTooBigOffers);

    const { fromToPriceList, isNoLiquidity, isRequiredMoreLiquidity } =
      getOffersExtendedUp(
        ALLOWED_OFFERS_BOUNDARY.UPPER,
        amountUndistributed,
        floorPrice,
        lowerFloorPrice,
        regularOffers,
        tooSmallOffers,
        MAX_OFFERS
      );

    expect(fromToPriceList).toStrictEqual([
      [800 * M, 937.44 * M],
      [806.5 * M, 944.88 * M],
      [813 * M, 952.32 * M],
      [819.5 * M, 959.76 * M],
      [826 * M, 967.2 * M],
    ]);
  });

  test("fp was decreased", () => {
    const amountUndistributed: number = 10_000 * M;
    const floorPrice: number = 970 * M;
    const priceList: string[] = [
      800 * M,
      806.5 * M,
      813 * M,
      819.5 * M,
      826 * M,
      832.5 * M,
      839 * M,
      845.5 * M,
      852 * M,
      858.5 * M,
      865 * M,
      871.5 * M,
      878 * M,
      884.5 * M,
      891 * M,
      897.5 * M,
      904 * M,
      910.5 * M,
      917 * M,
      923.5 * M,
      930 * M,
    ].map((x) => x.toString());
    const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
    const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
    const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList
    );

    const expectedRegularOffers: number[] = [
      897.5 * M,
      891 * M,
      884.5 * M,
      878 * M,
      871.5 * M,
      865 * M,
      858.5 * M,
      852 * M,
      845.5 * M,
      839 * M,
      832.5 * M,
      826 * M,
      819.5 * M,
      813 * M,
      806.5 * M,
      800 * M,
    ];
    const expectedTooSmallOffers: number[] = [];
    const expectedTooBigOffers: number[] = [
      930 * M,
      923.5 * M,
      917 * M,
      910.5 * M,
      904 * M,
    ];

    expect(regularOffers).toStrictEqual(expectedRegularOffers);
    expect(tooSmallOffers).toStrictEqual(expectedTooSmallOffers);
    expect(tooBigOffers).toStrictEqual(expectedTooBigOffers);

    const { fromToPriceList, isNoLiquidity } = getOffersExtendedDown(
      ALLOWED_OFFERS_BOUNDARY.LOWER,
      amountUndistributed,
      floorPrice,
      upperFloorPrice,
      regularOffers,
      tooBigOffers
    );

    expect(fromToPriceList).toStrictEqual([
      [930 * M, 795.2 * M],
      [923.5 * M, 790.4 * M],
      [917 * M, 785.6 * M],
      [910.5 * M, 780.8 * M],
      [904 * M, 776 * M],
    ]);
  });

  test("not enough liquidity to update prices", () => {
    const amountUndistributed: number = 600 * M;
    const floorPrice: number = 1_040 * M;
    const priceList: string[] = [
      800 * M,
      806.5 * M,
      813 * M,
      819.5 * M,
      826 * M,
      832.5 * M,
      839 * M,
      845.5 * M,
      852 * M,
      858.5 * M,
      865 * M,
      871.5 * M,
      878 * M,
      884.5 * M,
      891 * M,
      897.5 * M,
      904 * M,
      910.5 * M,
      917 * M,
      923.5 * M,
      930 * M,
    ].map((x) => x.toString());
    const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
    const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
    const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList
    );

    const { fromToPriceList, isNoLiquidity, isRequiredMoreLiquidity } =
      getOffersExtendedUp(
        ALLOWED_OFFERS_BOUNDARY.UPPER,
        amountUndistributed,
        floorPrice,
        lowerFloorPrice,
        regularOffers,
        tooSmallOffers,
        MAX_OFFERS
      );

    expect(fromToPriceList).toStrictEqual([]);
    expect(isRequiredMoreLiquidity).toStrictEqual(true);
  });

  test("no offers", () => {
    const floorPrice: number = 1_000 * M;
    const priceList: string[] = [];
    const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
    const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;
    const { regularOffers, tooSmallOffers, tooBigOffers } = splitOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList
    );

    const expectedRegularOffers: number[] = [];
    const expectedTooSmallOffers: number[] = [];
    const expectedTooBigOffers: number[] = [];

    expect(regularOffers).toStrictEqual(expectedRegularOffers);
    expect(tooSmallOffers).toStrictEqual(expectedTooSmallOffers);
    expect(tooBigOffers).toStrictEqual(expectedTooBigOffers);
  });
});
