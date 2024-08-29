import { getLast } from "../../common/utils";
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

  test("empty dataset, 4 samples window", () => {
    const samples: PriceSample[] = [];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [0, []];

    expect(calcTwap(window, samples)).toStrictEqual(expected);
  });

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
    const window: number = 48;
    const expected: number[] = [
      0, 100, 100, 105, 100, 103.75, 105, 101.25, 106.25, 107.5, 110, 115,
      117.5, 115, 125, 0,
    ];

    let twapList: number[] = [];
    for (let i = 0; i < 16; i++) {
      let samples = [...SAMPLES, NEW_SAMPLE].slice(Math.max(i - 5, 0), i);
      const newSample: PriceSample = getLast(samples);
      samples = samples.slice(0, -1);
      const [twap] = calcTwap(window, samples, newSample);
      twapList.push(twap);
    }

    expect(twapList).toStrictEqual(expected);
  });
});

describe("testing calcEma", () => {
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

  test("empty dataset, 4 samples window", () => {
    const samples: PriceSample[] = [];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [0, []];

    expect(calcEma(window, samples)).toStrictEqual(expected);
  });

  test("1 sample dataset, 4 samples window", () => {
    const samples: PriceSample[] = [];
    const newSample: PriceSample = SAMPLES[0];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      200,
      [{ value: 100, timestamp: 12 }],
    ];

    expect(calcEma(window, samples, newSample)).toStrictEqual(expected);
  });

  test("2 samples dataset, 4 samples window", () => {
    const samples = SAMPLES.slice(0, 1);
    const newSample: PriceSample = SAMPLES[1];
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      104,
      [
        { value: 100, timestamp: 12 },
        { value: 110, timestamp: 24 },
      ],
    ];

    expect(calcEma(window, samples, newSample)).toStrictEqual(expected);
  });

  test("default dataset, 4 samples window", () => {
    const samples = SAMPLES;
    const newSample: PriceSample = NEW_SAMPLE;
    const window: number = 48;
    const expected: [number, PriceSample[]] = [
      114,
      [
        { value: 105, timestamp: 60 },
        { value: 95, timestamp: 72 },
        { value: 110, timestamp: 84 },
        { value: 120, timestamp: 96 },
        { value: 115, timestamp: 108 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcEma(window, samples, newSample)).toStrictEqual(expected);
  });

  test("default dataset, 0.5 sample window", () => {
    const samples = SAMPLES;
    const newSample: PriceSample = NEW_SAMPLE;
    const window: number = 6;
    const expected: [number, PriceSample[]] = [
      131.111111,
      [
        { value: 120, timestamp: 102 },
        { value: 115, timestamp: 108 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcEma(window, samples, newSample)).toStrictEqual(expected);
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
      114.533333,
      [
        { value: 90, timestamp: 52 },
        { value: 115, timestamp: 55 },
        { value: 105, timestamp: 60 },
        { value: 95, timestamp: 61 },
        { value: 110, timestamp: 94 },
        { value: 120, timestamp: 96 },
        { value: 115, timestamp: 100 },
        { value: 125, timestamp: 120 },
      ],
    ];

    expect(calcEma(window, samples, newSample)).toStrictEqual(expected);
  });

  test("passing through default dataset, 4 samples window", () => {
    const window: number = 48;
    const expected: number[] = [
      0, 200, 164, 134.399999, 126.639999, 117.983999, 108.790399, 109.274239,
      113.564543, 114.138725, 118.483235, 121.089941, 122.653964, 123.592378,
      126.407622, 0,
    ];

    let tempEma: number | undefined = undefined;
    let emaList: number[] = [];
    for (let i = 0; i < 16; i++) {
      let samples = [...SAMPLES, NEW_SAMPLE].slice(Math.max(i - 5, 0), i);
      const newSample: PriceSample = getLast(samples);
      samples = samples.slice(0, -1);
      const [ema] = calcEma(window, samples, newSample, tempEma);
      tempEma = ema;
      emaList.push(ema);
    }

    expect(emaList).toStrictEqual(expected);
  });
});
