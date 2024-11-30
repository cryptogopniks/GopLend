import { floor } from "../../common/utils";
import { calcCollectionPrice, PriceSample } from "../services/math";

describe("testing calcCollectionPrice", () => {
  const TWAP_WEIGHT: number = 0.7;
  const BLOCK_TIME: number = 6;
  const SAMPLING_WINDOW: number = 10 * BLOCK_TIME;
  const FP_TWAP_WINDOW_MULTIPLIER: number = 2;
  const FLOOR_PRICE_BOUNDARY = {
    UPPER: 0.93,
    LOWER: 0.8,
  };

  test("adding first low offer when FP is set", () => {
    const floorPriceSampleListDataset: PriceSample[] = [
      { value: 400, timestamp: 12 },
      { value: 400, timestamp: 24 },
      { value: 400, timestamp: 36 },
      { value: 400, timestamp: 48 },
      { value: 400, timestamp: 60 },
      { value: 400, timestamp: 72 },
      { value: 400, timestamp: 84 },
      { value: 400, timestamp: 96 },
      { value: 400, timestamp: 108 },
      { value: 400, timestamp: 120 },
      { value: 400, timestamp: 132 },
      { value: 400, timestamp: 144 },
      { value: 400, timestamp: 156 },
      { value: 400, timestamp: 168 },
      { value: 400, timestamp: 180 },
      { value: 400, timestamp: 192 },
      { value: 400, timestamp: 204 },
      { value: 400, timestamp: 216 },
      { value: 400, timestamp: 228 },
      { value: 400, timestamp: 240 },
    ];

    const offerPriceListDataset: number[][] = [
      [],
      [],
      [],
      [],
      [],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
      [325],
    ];

    const expected: number[] = [
      346, 346, 346, 345.999, 345.999, +344.199, 339.207, 336.054, 330.235,
      329.024, 325.75, 325.519, 325.359, 325.249, 325.172, 325.119, 325.082,
      325.057, 325.039, 325.027,
    ];

    let floorPriceSampleList: PriceSample[] = [];
    let prevFloorPriceEma: number = floorPriceSampleListDataset[0].value;
    let meanOfferSampleList: PriceSample[] = [];
    let prevMeanOfferEma: number = 0;
    let collectionPriceList: number[] = [];

    for (let i = 0; i < floorPriceSampleListDataset.length; i++) {
      const newFloorPriceSample = floorPriceSampleListDataset[i];
      const offerPriceList = offerPriceListDataset[i];
      const {
        twapSampleList,
        floorPriceEma,
        twapOfferList,
        offerEma,
        collectionPrice,
      } = calcCollectionPrice(
        FLOOR_PRICE_BOUNDARY.UPPER,
        FLOOR_PRICE_BOUNDARY.LOWER,
        TWAP_WEIGHT,
        SAMPLING_WINDOW,
        FP_TWAP_WINDOW_MULTIPLIER,
        floorPriceSampleList,
        newFloorPriceSample,
        prevFloorPriceEma,
        offerPriceList,
        meanOfferSampleList,
        prevMeanOfferEma
      );

      floorPriceSampleList = twapSampleList;
      prevFloorPriceEma = floorPriceEma;
      meanOfferSampleList = twapOfferList;
      prevMeanOfferEma = offerEma;
      collectionPriceList = [...collectionPriceList, floor(collectionPrice, 3)];
    }

    expect(collectionPriceList).toStrictEqual(expected);
  });
});
