import { calcTwap, PriceSample } from "../services/math";

describe("testing calcTwap", () => {
  test("default dataset, 4 samples window", () => {
    const window: number = 48;
    const samples: PriceSample[] = [
      { value: 100, timestamp: 12 },
      { value: 110, timestamp: 24 },
      { value: 90, timestamp: 36 },
      { value: 115, timestamp: 48 },
      { value: 105, timestamp: 60 },
      { value: 95, timestamp: 72 },
      { value: 110, timestamp: 84 },
      { value: 120, timestamp: 96 },
      { value: 115, timestamp: 108 },
      { value: 125, timestamp: 120 },
    ];
    const expected: [number, PriceSample[]] = [
      110,
      [
        { value: 95, timestamp: 72 },
        { value: 110, timestamp: 84 },
        { value: 120, timestamp: 96 },
        { value: 115, timestamp: 108 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });
});
