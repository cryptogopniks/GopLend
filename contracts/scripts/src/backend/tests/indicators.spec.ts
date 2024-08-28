import { calcTwap, PriceSample } from "../services/math";

describe("testing calcTwap", () => {
  const SAMPLES: PriceSample[] = [
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

  test("empty dataset, 4 samples window", () => {
    const samples: PriceSample[] = [];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [0, []];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });

  test("1 sample dataset, 4 samples window", () => {
    const samples = SAMPLES.slice(0, 1);
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      100,
      [{ value: 100, timestamp: 12 }],
    ];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });

  test("2 samples dataset, 4 samples window", () => {
    const samples = SAMPLES.slice(0, 2);
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      100,
      [
        { value: 100, timestamp: 12 },
        { value: 110, timestamp: 24 },
      ],
    ];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });

  test("default dataset, 4 samples window", () => {
    const samples = SAMPLES;
    const window: number = 48;
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

  test("default dataset, 0.5 sample window", () => {
    const samples = SAMPLES;
    const window: number = 6;
    const expected: [number, PriceSample[]] = [
      115,
      [
        { value: 115, timestamp: 114 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });

  test("unequal periods dataset, 4 samples window", () => {
    const samples: PriceSample[] = [
      { value: 100, timestamp: 2 },
      { value: 110, timestamp: 24 },
      { value: 90, timestamp: 28 },
      { value: 115, timestamp: 55 },
      { value: 105, timestamp: 60 },
      { value: 95, timestamp: 61 },
      { value: 110, timestamp: 94 },
      { value: 120, timestamp: 96 },
      { value: 115, timestamp: 100 },
      { value: 125, timestamp: 120 },
    ];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      106.04166666666667,
      [
        { value: 95, timestamp: 72 },
        { value: 110, timestamp: 94 },
        { value: 120, timestamp: 96 },
        { value: 115, timestamp: 100 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });

  test("passing through default dataset, 4 samples window", () => {
    const window: number = 48;
    const expected: number[] = [
      0, 100, 100, 105, 100, 103.75, 105, 101.25, 106.25, 107.5, 110, 115,
      117.5, 115, 125, 0,
    ];

    let twapList: number[] = [];
    for (let i = 0; i < 16; i++) {
      const samples = SAMPLES.slice(Math.max(i - 5, 0), i);
      const [twap] = calcTwap(window, samples);
      twapList.push(twap);
    }

    expect(twapList).toStrictEqual(expected);
  });
});
