import { splitOffers, calcOffers } from "../services/math";

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

describe("testing calcOffers", () => {
  test("unchanged price", () => {
    const maxOffers: number = 21;
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

    const offers = calcOffers(
      lowerFloorPrice,
      upperFloorPrice,
      [],
      18_165 * M,
      maxOffers
    );

    expect(offers).toStrictEqual(expectedRegularOffers);
  });

  test("unchanged price, offers limit", () => {
    const maxOffers: number = 21;
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

    const offers = calcOffers(
      lowerFloorPrice,
      upperFloorPrice,
      [],
      181_650 * M,
      maxOffers
    );

    expect(offers).toStrictEqual(expectedRegularOffers);
  });

  test("unchanged price, limited liquidity", () => {
    const maxOffers: number = 21;
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

    const expectedRegularOffers: number[] = [930 * M, 865 * M, 800 * M];

    const offers = calcOffers(
      lowerFloorPrice,
      upperFloorPrice,
      [],
      3_000 * M,
      maxOffers
    );

    expect(offers).toStrictEqual(expectedRegularOffers);
  });

  test("recovering random offers", () => {
    const maxOffers: number = 21;
    const floorPrice: number = 1_000 * M;
    const priceList: string[] = [
      800 * M,
      // 806.5 * M,
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
      // 923.5 * M,
      930 * M,
    ].map((x) => x.toString());
    const lowerFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.LOWER;
    const upperFloorPrice = floorPrice * FLOOR_PRICE_BOUNDARY.UPPER;

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

    const offers = calcOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList,
      1_730 * M,
      maxOffers
    );

    expect(offers).toStrictEqual(expectedRegularOffers);
  });

  test("fp was increased", () => {
    const maxOffers: number = 21;
    const amountUndistributed: number = 250_000 * M;
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
    const expectedOffers: number[] = [
      967.2 * M,
      960.44 * M,
      953.68 * M,
      946.92 * M,
      940.16 * M,
      933.4 * M,
      926.64 * M,
      919.88 * M,
      913.12 * M,
      906.36 * M,
      899.6 * M,
      892.84 * M,
      886.08 * M,
      879.32 * M,
      872.56 * M,
      865.8 * M,
      859.04 * M,
      852.28 * M,
      845.52 * M,
      838.76 * M,
      832.0 * M,
    ];

    expect(regularOffers).toStrictEqual(expectedRegularOffers);
    expect(tooSmallOffers).toStrictEqual(expectedTooSmallOffers);
    expect(tooBigOffers).toStrictEqual(expectedTooBigOffers);

    const offers = calcOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList,
      amountUndistributed,
      maxOffers
    );

    expect(offers).toStrictEqual(expectedOffers);
  });

  test("fp was decreased", () => {
    const maxOffers: number = 21;
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
    const expectedOffers: number[] = [
      902.1 * M,
      895.795 * M,
      889.49 * M,
      883.185 * M,
      876.88 * M,
      870.575 * M,
      864.27 * M,
      857.965 * M,
      851.66 * M,
      845.355 * M,
      839.05 * M,
      832.745 * M,
      826.44 * M,
      820.135 * M,
      813.83 * M,
      807.525 * M,
      801.22 * M,
      794.915 * M,
      788.61 * M,
      782.305 * M,
      776.0 * M,
    ];

    expect(regularOffers).toStrictEqual(expectedRegularOffers);
    expect(tooSmallOffers).toStrictEqual(expectedTooSmallOffers);
    expect(tooBigOffers).toStrictEqual(expectedTooBigOffers);

    const offers = calcOffers(
      lowerFloorPrice,
      upperFloorPrice,
      priceList,
      amountUndistributed,
      maxOffers
    );

    expect(offers).toStrictEqual(expectedOffers);
  });
});
