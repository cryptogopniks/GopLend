interface Pair {
  A: number;
  B: number;
}

function groupPairs(pairs: Pair[], C: number, M?: number): Pair[][] {
  // Sort pairs descending by A
  pairs.sort((a, b) => b.A - a.A);

  const groups: Pair[][] = [];

  for (const pair of pairs) {
    let placed = false;
    // Try to fit the pair into an existing group
    for (const group of groups) {
      const groupSum = group.reduce((sum, p) => sum + p.B, 0);
      if (groupSum + pair.B <= C) {
        group.push(pair);
        placed = true;
        break;
      }
    }
    // If the pair doesn't fit in any existing group, create a new group
    if (!placed) {
      groups.push([pair]);
    }

    // If M is specified and we have created M groups, stop adding new groups
    if (M !== undefined && groups.length === M) {
      break;
    }
  }

  // If we have more pairs, try to fit them into existing groups
  if (
    M !== undefined &&
    pairs.length > groups.reduce((sum, group) => sum + group.length, 0)
  ) {
    for (const pair of pairs) {
      if (groups.some((group) => group.includes(pair))) continue;
      for (const group of groups) {
        const groupSum = group.reduce((sum, p) => sum + p.B, 0);
        if (groupSum + pair.B <= C) {
          group.push(pair);
          break;
        }
      }
    }
  }

  return groups;
}

// Test cases
function runTests() {
  const testCases = [
    {
      pairs: [
        { A: 5, B: 2 },
        { A: 3, B: 3 },
        { A: 7, B: 1 },
        { A: 2, B: 4 },
        { A: 6, B: 2 },
      ],
      C: 5,
      expected: [
        [
          { A: 7, B: 1 },
          { A: 3, B: 3 },
        ],
        [
          { A: 6, B: 2 },
          { A: 5, B: 2 },
        ],
        [{ A: 2, B: 4 }],
      ],
    },
    {
      pairs: [
        { A: 10, B: 1 },
        { A: 8, B: 2 },
        { A: 6, B: 3 },
        { A: 4, B: 4 },
        { A: 2, B: 5 },
      ],
      C: 6,
      expected: [
        [
          { A: 10, B: 1 },
          { A: 4, B: 4 },
        ],
        [
          { A: 8, B: 2 },
          { A: 6, B: 3 },
        ],
        [{ A: 2, B: 5 }],
      ],
    },
    {
      pairs: [
        { A: 1, B: 1 },
        { A: 2, B: 2 },
        { A: 3, B: 3 },
      ],
      C: 10,
      expected: [
        [
          { A: 3, B: 3 },
          { A: 2, B: 2 },
          { A: 1, B: 1 },
        ],
      ],
    },
    {
      pairs: [
        { A: 10, B: 1 },
        { A: 9, B: 2 },
        { A: 8, B: 3 },
        { A: 7, B: 4 },
        { A: 6, B: 5 },
      ],
      C: 5,
      M: 3,
      expected: [
        [
          { A: 10, B: 1 },
          { A: 7, B: 4 },
        ],
        [
          { A: 9, B: 2 },
          { A: 8, B: 3 },
        ],
        [{ A: 6, B: 5 }],
      ],
    },
    {
      pairs: [
        { A: 12, B: 1 },
        { A: 11, B: 2 },
        { A: 10, B: 3 },
        { A: 9, B: 4 },
        { A: 8, B: 5 },
        { A: 7, B: 1 },
        { A: 6, B: 2 },
        { A: 5, B: 3 },
        { A: 4, B: 4 },
      ],
      C: 5,
      M: 3,
      expected: [
        [
          { A: 12, B: 1 },
          { A: 9, B: 4 },
        ],
        [
          { A: 11, B: 2 },
          { A: 10, B: 3 },
        ],
        [{ A: 8, B: 5 }],
      ],
    },
  ];

  testCases.forEach((testCase, index) => {
    const result = groupPairs(testCase.pairs, testCase.C, testCase.M);
    console.log(`Test case ${index + 1}:`);
    console.log("Result:", result);
    console.log("Expected:", testCase.expected);
    console.log(
      "Passed:",
      JSON.stringify(result) === JSON.stringify(testCase.expected)
    );
    console.log("---");
  });
}

runTests();
