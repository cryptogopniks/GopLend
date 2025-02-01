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

  // SG API returns data in wrong order, we need to fix it
  return addressList.map((address) => {
    const fpItem = data.collections.collections.find(
      (x) => x.contractAddress === address
    );

    return {
      collection: address,
      price: floor(fpItem?.floor?.amountUsd || 0, 2),
    };
  });
}
