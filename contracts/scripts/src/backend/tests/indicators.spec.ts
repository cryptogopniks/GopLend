import { calcEma, calcTwap, PriceSample } from "../services/math";

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
  ];
  const NEW_SAMPLE: PriceSample = { value: 125, timestamp: 120 };

  test("1 sample dataset, 4 samples window", () => {
    const samples: PriceSample[] = [];
    const newSample: PriceSample = SAMPLES[0];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      100,
      [{ value: 100, timestamp: 12 }],
    ];

    expect(calcTwap(window, samples, newSample)).toStrictEqual(expected);
  });

  test("2 samples dataset, 4 samples window", () => {
    const samples = SAMPLES.slice(0, 1);
    const newSample: PriceSample = SAMPLES[1];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      100,
      [
        { value: 100, timestamp: 12 },
        { value: 110, timestamp: 24 },
      ],
    ];

    expect(calcTwap(window, samples, newSample)).toStrictEqual(expected);
  });

  test("default dataset, 4 samples window", () => {
    const samples = SAMPLES;
    const newSample: PriceSample = NEW_SAMPLE;
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

    expect(calcTwap(window, samples, newSample)).toStrictEqual(expected);
  });

  test("default dataset, 0.5 sample window", () => {
    const samples = SAMPLES;
    const newSample: PriceSample = NEW_SAMPLE;
    const window: number = 6;
    const expected: [number, PriceSample[]] = [
      115,
      [
        { value: 115, timestamp: 114 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcTwap(window, samples, newSample)).toStrictEqual(expected);
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
    ];
    const newSample: PriceSample = NEW_SAMPLE;
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      106.041666,
      [
        { value: 95, timestamp: 72 },
        { value: 110, timestamp: 94 },
        { value: 120, timestamp: 96 },
        { value: 115, timestamp: 100 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcTwap(window, samples, newSample)).toStrictEqual(expected);
  });

  test("passing through default dataset, 4 samples window", () => {
    const dataSet: PriceSample[] = [
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
      { value: 0, timestamp: 132 },
      { value: 0, timestamp: 144 },
      { value: 0, timestamp: 156 },
      { value: 0, timestamp: 168 },
      { value: 0, timestamp: 180 },
    ];

    const window: number = 48;
    const expected: number[] = [
      100, 100, 105, 100, 103.75, 105, 101.25, 106.25, 107.5, 110, 117.5, 90,
      60, 31.25, 0,
    ];

    let samples: PriceSample[] = [];
    let twapList: number[] = [];
    let twap: number | undefined = undefined;

    for (let i = 0; i < dataSet.length; i++) {
      [twap, samples] = calcTwap(window, samples, dataSet[i]);
      twapList.push(twap);
    }

    expect(twapList).toStrictEqual(expected);
  });
});

describe("testing calcEma", () => {
  test("passing through default dataset, 4 samples window", () => {
    const dataSet: PriceSample[] = [
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
      { value: 0, timestamp: 132 },
      { value: 0, timestamp: 144 },
      { value: 0, timestamp: 156 },
      { value: 0, timestamp: 168 },
      { value: 0, timestamp: 180 },
      { value: 0, timestamp: 192 },
      { value: 0, timestamp: 204 },
      { value: 0, timestamp: 216 },
      { value: 0, timestamp: 228 },
      { value: 0, timestamp: 240 },
    ];

    const windowSamples: number = 4;
    const expected: number[] = [
      100, 104, 98.4, 105.039999, 105.023999, 101.014399, 104.608639,
      110.765183, 112.459109, 117.475465, 70.485278, 42.291166, 25.374699,
      15.224819, 9.134891, 5.480934, 3.28856, 1.973135, 1.183881, 0.710328,
    ];

    let emaList: number[] = [];
    let ema: number | undefined = undefined;

    for (let i = 0; i < dataSet.length; i++) {
      ema = calcEma(windowSamples, dataSet[i].value, ema);
      emaList.push(ema);
    }

    expect(emaList).toStrictEqual(expected);
  });
});
