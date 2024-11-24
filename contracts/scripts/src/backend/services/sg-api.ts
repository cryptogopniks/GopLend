import { floor } from "../../common/utils";

interface FloorPricesResponse {
  collections: {
    collections: { name: string; floor: { amountUsd: number } }[];
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
          name
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
    collection: x.name,
    price: floor(x.floor.amountUsd, 2),
  }));
}
