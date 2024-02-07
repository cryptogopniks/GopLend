<<<<<<< HEAD
//import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { getSigner } from "../account/signer";
//import { getSeed } from "./seed/get-seed";
=======
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { getSigner } from "../account/signer";
import { getSeed } from "./seed/get-seed";
>>>>>>> 3326615395d460d4f0d981c76225408b8dc2a9ca
import { l } from "../../common/utils";
import { chains } from "chain-registry";
import { readFile } from "fs/promises";
import { getCwExecHelpers } from "../../common/account/cw-helpers";
import { ApolloClient, InMemoryCache, createHttpLink, gql } from '@apollo/client';
import { HttpLink } from '@apollo/client';
import fetch from 'cross-fetch';

const client = new ApolloClient({
  link: new HttpLink({ uri: 'https://galaxy-graphql-testnet.fly.dev/graphql', fetch }),
  cache: new InMemoryCache(),
});


var address = "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw"
var amounts

const getTraitsAndOwner = async (address: string) => {
  const { data } = await client.query({
    query: gql`
      query Collection($address: String!) {
        collection(address: $address) {
          offers {
            offerPrice {
              amount
            }
          }
        }
      }
    `,
    variables: {
      address: address,
    }
  });
  return data;
};


<<<<<<< HEAD
let data: any;
const getData = async () => {
  try {
    const res1 = await getTraitsAndOwner(address);
    const amounts: any = res1.collection.offers.map((offer: { offerPrice: { amount: any; }; }) => offer.offerPrice.amount);
    
    data = [
=======

const getData = async () => {
  try {
    const res1 = await getTraitsAndOwner(address);
    console.log(res1);
    console.log(JSON.stringify(res1, null, 2));
    amounts = res1.collection.offers.map((offer: { offerPrice: { amount: any; }; }) => offer.offerPrice.amount);
    console.log(amounts);

    const data = [
>>>>>>> 3326615395d460d4f0d981c76225408b8dc2a9ca
      {
        collection_address:
          "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw",
        currency: { token: { native: { denom: "ustars" } }, decimals: 6 },
        raw_prices: amounts,
        trading_volume_per_day: "1000",
      },
    ];

<<<<<<< HEAD
    return data;
=======
    console.log(data);
>>>>>>> 3326615395d460d4f0d981c76225408b8dc2a9ca

  } catch (error) {
    console.log(error);
  }
};

<<<<<<< HEAD
const fetchData = async () => {
  try {
    const fetchedData = await getData();
    console.log(fetchedData);
    if (fetchedData) {
      async function updateCW() {
          const newFileContent = await readFile(`./wallets/0_key.json`, { encoding: "utf8" });
          const jsonContent = JSON.parse(newFileContent);
          const seed = jsonContent.mnemonic.trim();
          if (!seed) throw new Error("Seed is not found!");
          const chain = chains.find((x) => x.chain_id == CHAIN_ID);
          if (!chain) throw new Error(`${CHAIN_ID} config is not found!`);
          const gasPrice = "1ustars"
          const { signer, owner } = await getSigner(PREFIX, seed);
          const { cwUpdatePrices } = await getCwExecHelpers(CHAIN_ID, RPC, owner, signer);
      
                try {
                  const res = await cwUpdatePrices(
                    data,
                    gasPrice
                  );
                  if (res) {
                    l(res);
                  }
                } catch (error) {
                  l(error);
                }
              }
      updateCW();
      }
  } catch (error) {
    console.log(error);
  }
};

fetchData();


=======
getData();

var data = [
        {
           collection_address:
             "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw",
           currency: { token: { native: { denom: "ustars" } }, decimals: 6 },
           raw_prices: amounts,
           trading_volume_per_day: "1000",
         },
       ]

l(data)
>>>>>>> 3326615395d460d4f0d981c76225408b8dc2a9ca

// network config
const CHAIN_ID = "elgafar-1";
const PREFIX = "stars";
const RPC = "https://rpc.elgafar-1.stargaze-apis.com";



<<<<<<< HEAD
=======



async function updateCW() {
    const newFileContent = await readFile(`./wallets/0_key.json`, { encoding: "utf8" });
    const jsonContent = JSON.parse(newFileContent);
    const seed = jsonContent.mnemonic.trim();
    if (!seed) throw new Error("Seed is not found!");
    const chain = chains.find((x) => x.chain_id == CHAIN_ID);
    if (!chain) throw new Error(`${CHAIN_ID} config is not found!`);
    const gasPrice = "1ustars"
    const { signer, owner } = await getSigner(PREFIX, seed);
    const { cwUpdatePrices } = await getCwExecHelpers(CHAIN_ID, RPC, owner, signer);

          try {
            const res = await cwUpdatePrices(
              data,
              gasPrice
            );
            if (res) {
              l(res);
            }
          } catch (error) {
            l(error);
          }
        }
updateCW();
>>>>>>> 3326615395d460d4f0d981c76225408b8dc2a9ca
