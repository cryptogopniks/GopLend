//import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { getSigner } from "../account/signer";
//import { getSeed } from "./seed/get-seed";
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


  // API шаблоны

const getFP = async (address: string) => {
    const { data } = await client.query({
      query: gql`
      query Query($address: String!) {
          collection(address: $address) {
            floorPrice
          }
        }
        
      `,
      variables: {
        address: address,
      }
    });
    return data;
  };


const getOffer = async (address: string) => {
    const { data } = await client.query({
      query: gql`
      query Query($address: String!) {
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


  const updateAll = async (LTV: string) => {

    let data: any;
    const CHAIN_ID = "elgafar-1";
    const PREFIX = "stars";
    const RPC = "https://rpc.elgafar-1.stargaze-apis.com";
    
    
    const fetchData = async () => {
      try {
        if (LTV) {
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
      }}};

/*
Хм, я думаю, что к нашей формуле можно подключить FP.

Действительно, если оффер на бэд кидсов 10к старс, а FP 100к, то он просто должен быть не действительным.

Можно попробовать пойти другим путем

Представим, что у нас LTV не может превышать 50%. Ликвидация начинается, если максимальный оффер на коллекцию, меньше, чем сумма, которую мы взяли в кредит, но только если он состовляет не менее 60% от FP, чтобы ограничить возможность абуза


FP упал на 10% буквально за 2 минуты. Теоретические действия протокола:

1) Если заняли < 0.5 * FP (LTV)

Например, заняли 10к, а LTV в текущий момент снизился с 50к к 45к. Ничего страшного, ликвидации нет

2) Если заняли > 0.5 * FP (LTV)

Проводим 1 первичную ликвидацию* 

мы дегены и заняли на 48% от FP

LTV у нас 45к, а заняли мы 48к. FP на нфт при этом 90к. 

Нам нужно срочно найти оффера на Stargaze или нашем протоколе для ликвидаций (нужно продать как можно выгоднее)

Если есть положительный оффер (больше 48к), то продаем

___

Что такое 1-й шаг ликвидации (первичная ликивдация)? Возможно, из-за того, что кто-то выставил своего бэд кидса за 10к, при том, что FP 100к, наш протоколо ахереет. 

Поэтому, нам нужно проверить, купят ли этого бэд кидса в течении какого-либо времени (Например, 1 час)

И в этот час мы просто блокируем возможность вывода NFT из злога, но не ликвидируем*/



// Сначала определим LTV

// LTV = 0.5 * FP, но не простой FP, а то можно рект поймать

// Не простой FP нужно определить не только по его цене, но и по тому, как долго он находится на рынке.
// Так, если FP на минуту снизился на 30%, то мы не должны это считать. Тут расчеты довольно сложные. Так как простым вычислением не понять, манипулятивно опускается FP или из-за движения рынка.
// Поэтому желательно учесть не только время, но и количество NFT близких к FP а также больше ли оффер чем FP

//Чтобы не запутаться, начнем с простого, просто определим FP

var addr = "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw"
var FP = await getFP(addr);
var LTV

// Далее, учтем время, на которое сохраняется наш FP

var manip = false // Пусть будем считать, что снижение FP не манипулятивное

async function checkOutput() {
    let fpReached = false;
    let fpOutput;
  
    // Функция, которую необходимо проверить
    async function getFP(addr: string): Promise<any> {
      return fpOutput;
    }
  
    // Проверяем вывод каждые 10 секунд
    const interval = setInterval(async () => {
      const output = await getFP(addr);
      if (!fpReached) {
        fpOutput = output;
        fpReached = true;
        console.log(fpOutput);
      } else {
        if (output !== fpOutput) {
          console.log('⛔️ Вывод не соответствует FP:', output);
          console.log('Скорее всего была обычная манипуляция');
          manip = true // Это манимпуляция
          clearInterval(interval);
        } else {
          console.log('✅ Вывод соответствует FP:', output);
        }
      }
    }, 10000);
  
    // Завершаем проверку через 2 минуты (условные)
    setTimeout(() => {
      console.log('Емае, реально FP просел, и не откупают');
      clearInterval(interval);
      console.log('Проверка завершена');
    }, 120000);
  }
  
  checkOutput();

  if (manip){
    // Можно не обновлять LTV
  }else{
    // Нужно проверить, ниже ли FP, чем оффера 
    var offer = await getOffer(addr)
    if (offer > FP){
    }else{
        // Все таки, нужно обновлять LTV, так
        LTV = 0.5 * FP
        updateAll(LTV)
    }
  }


  // Получилась достаточно простая проверка на манипуляции





