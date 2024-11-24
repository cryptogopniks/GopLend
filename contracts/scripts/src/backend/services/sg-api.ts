import { GraphQLClient, gql } from "graphql-request";
// import { GraphQLClient, gql } from "graphql-request/build/entrypoints/main.js";

// const client = new ApolloClient({
//   uri: "https://graphql.mainnet.stargaze-apis.com/graphql",
//   cache: new InMemoryCache(),
// });

// export async function getFloorPrices(addressList: string[]) {
//   const { data } = await client.query({
//     query: gql`
//       query Collections($filterByAddrs: [String!]) {
//         collections(filterByAddrs: $filterByAddrs) {
//           collections {
//             name
//             floor {
//               amountUsd
//             }
//           }
//         }
//       }
//     `,
//     variables: {
//       addressList,
//     },
//     fetchPolicy: "cache-first",
//   });

//   return data;
// }

// const document = gql`
//   {
//     company {
//       ceo
//     }
//   }
// `;

// export async function getFloorPrices(_addressList: string[]) {
//   return await request(
//     "https://graphql.mainnet.stargaze-apis.com/graphql",
//     document,
//     { arguments: {} }
//   );
// }

const client = new GraphQLClient(
  "https://graphql.mainnet.stargaze-apis.com/graphql"
);

export async function getFloorPrices(addressList: string[]) {
  const query = gql`
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

  const variables = {
    filterByAddrs: addressList,
  };

  const data = await client.request(query, variables);

  return data;
}
