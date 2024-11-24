import { floor } from "../../common/utils";

interface FloorPricesResponse {
  collections: {
    collections: { contractAddress: string; floor: { amountUsd: number } }[];
  };
}

interface CollectionInfo {
  collection: string;
  price: number;
}

export async function getFloorPrices(
  addressList: string[]
): Promise<CollectionInfo[]> {
  const query = `
    query Collections($filterByAddrs: [String!]) {
      collections(filterByAddrs: $filterByAddrs) {
        collections {
          contractAddress
          floor {
            amountUsd
          }
        }
      }
    }
  `;

  const response = await fetch(
    "https://graphql.mainnet.stargaze-apis.com/graphql",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        query,
        variables: {
          filterByAddrs: addressList,
        },
      }),
    }
  );

  const { data }: { data: FloorPricesResponse } = await response.json();

  return data.collections.collections.map((x) => ({
    collection: x.contractAddress,
    price: floor(x.floor.amountUsd, 2),
  }));
}
