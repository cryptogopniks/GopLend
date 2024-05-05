import { getSigner } from "../account/signer";
import { l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ADDRESS } from "../../common/config";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
} from "./utils";
import {
  getChainOptionById,
  getContractByWasm,
} from "../../common/config/config-utils";
import {
  getSgQueryHelpers,
  getSgExecHelpers,
} from "../../common/account/sg-helpers";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";

const COLLECTION = {
  BANDIT_1: "kujira18wpqlgsl9q8e707yp4gg4ugpy0ccnjq06hr7ggav0m4asdtceuzq7y2mps",
  BANDIT_2: "kujira1yp4vwjypr0zdcanltz0kgvt05kx3gvlzllge6rhffdn2j49rcauqw8u50c",
  BANDIT_3: "kujira1lnafxjcagzt4qlztarxskpgrlny9d936vmvsx4m4qv0mengwlyes5q84p0",
};

const TOKEN = {
  KUJI: "ukuji",
  BGL_KUJI:
    "factory/kujira1h85yg8utphr7qzu560u0ec682qfxvxq5tujt7md9ndagncxdf7qspjaxzs/bglKUJI",
  TEST: "factory/kujira1hywaw7mzn89temhe0u0gyyfs7uc6hlj098a7lxt8cuhdsgyehwtshl5dqg/test",
};

async function main() {
  try {
    const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
    const { chainId } = parseStoreArgs();
    const {
      PREFIX: PREFIX,
      OPTION: {
        RPC_LIST: [RPC],
        DENOM: DENOM,
        CONTRACTS: CONTRACTS,
        GAS_PRICE_AMOUNT: GAS_PRICE_AMOUNT,
        TYPE: TYPE,
      },
    } = getChainOptionById(CHAIN_CONFIG, chainId);

    const LENDING_PLATFORM_CONTRACT = getContractByWasm(
      CONTRACTS,
      "lending_platform.wasm"
    );
    const ORACLE_CONTRACT = getContractByWasm(CONTRACTS, "oracle.wasm");
    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

    const sgQueryHelpers = await getSgQueryHelpers(RPC);
    const sgExecHelpers = await getSgExecHelpers(RPC, owner, signer);

    const { adapter, lending, minter, oracle, utils, minterKujira } =
      await getCwQueryHelpers(chainId, RPC);
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const { getAllBalances } = sgQueryHelpers;
    const { sgMultiSend } = sgExecHelpers;

    const updateConfigs = async () => {
      // setup minter-kujira
      await h.minterKujira.cwCreateNative(
        LENDING_PLATFORM_CONTRACT.ADDRESS,
        "bglKUJI",
        6,
        100_000_000,
        TOKEN.KUJI,
        gasPrice
      );

      const [bglCurrency] = (
        await minterKujira.cwQueryCurrenciesByCreator(
          LENDING_PLATFORM_CONTRACT.ADDRESS
        )
      ).currencies;
      const bglKUJI = (bglCurrency as any)?.token?.native?.denom || "";
      l({ bglKUJI });

      // setup lending
      await h.lending.cwUpdateCommonConfig(
        {
          bglCurrency: {
            token: { native: { denom: bglKUJI } },
            decimals: 6,
          },
          collateralMinValue: 100,
          unbondingPeriod: 60,
          priceUpdatePeriod: 7 * 24 * 3600,
        },
        gasPrice
      );

      // deposit reserve liquidity
      await h.lending.cwDepositReserveLiquidity(
        1_000_000_000,
        { native: { denom: TOKEN.KUJI } },
        gasPrice
      );

      // add collections
      const collections = [
        [COLLECTION.BANDIT_1, "BANDIT_1"],
        [COLLECTION.BANDIT_2, "BANDIT_2"],
        [COLLECTION.BANDIT_3, "BANDIT_3"],
      ];

      let i = 0;

      for (const [collection_address, name] of collections) {
        await h.lending.cwCreateProposal(
          {
            proposal_type: {
              add_collection: {
                collection: { name, owner },
                collection_address,
              },
            },
            listing_price: {
              amount: "1",
              currency: {
                token: { native: { denom: TOKEN.KUJI } },
                decimals: 6,
              },
            },
          },
          gasPrice
        );

        await h.lending.cwAcceptProposal(
          ++i,
          1,
          { native: { denom: TOKEN.KUJI } },
          gasPrice
        );
      }
    };

    // await updateConfigs();
    // return;

    // await lending.cwQueryCollectionList();

    // await lending.cwQueryAddressConfig();
    // await lending.cwQueryRateConfig();
    // await lending.cwQueryCommonConfig();

    // await h.minterKujira.cwCreateNative(
    //   owner,
    //   "test",
    //   6,
    //   100_000_000,
    //   "ukuji",
    //   gasPrice
    // );
    // await minterKujira.cwQueryCurrenciesByCreator(owner);

    // await h.minterKujira.cwMint(
    //   420,
    //   owner,
    //   { native: { denom: TOKEN.TEST } },
    //   gasPrice
    // );
    // await getAllBalances(owner);

    // await h.lending.cwDeposit(
    //   10_000,
    //   { native: { denom: TOKEN.KUJI } },
    //   gasPrice
    // );
    // await getAllBalances(owner);

    // await h.lending.cwUnbond(
    //   10_000,
    //   { native: { denom: TOKEN.BGL_KUJI } },
    //   gasPrice
    // );
    // await lending.cwQueryUnbonder(owner);

    // await wait(60_000);
    // await h.lending.cwWithdraw(gasPrice);
    // await getAllBalances(owner);

    // await minterKujira.cwQueryCurrenciesByCreator(
    //   LENDING_PLATFORM_CONTRACT.ADDRESS
    // );

    // await updateConfigs();

    // await utils.cwQueryBalanceInNft(owner, COLLECTION.BANDIT_3); // 2,3 - [ '5', '6', '7', '8' ]
    // await utils.cwQueryOperators(owner, COLLECTION.BANDIT_1);

    // await lending.cwQueryCollectionList();

    await lending.cwQueryCollateralByOwner(owner);

    // await h.lending.cwApproveAndDepositCollateral(
    //   owner,
    //   LENDING_PLATFORM_CONTRACT.ADDRESS,
    //   [
    //     {
    //       collection_address: COLLECTION.BANDIT_2,
    //       token_id_list: ["5"],
    //     },
    //   ],
    //   gasPrice
    // );

    await h.lending.cwWithdrawCollateral(
      [
        {
          collection_address: COLLECTION.BANDIT_2,
          token_id_list: ["5"],
        },
      ],
      gasPrice
    );

    await lending.cwQueryCollateralByOwner(owner);

    // await oracle.cwQueryPrices();

    // await getAllBalances(ADDRESS.ORACLE_WORKER);
    // await getAllBalances(ADDRESS.LIQUIDATION_WORKER);
    // await sgMultiSend(
    //   "ukuji",
    //   [
    //     [ADDRESS.ORACLE_WORKER, 1],
    //     [ADDRESS.LIQUIDATION_WORKER, 2],
    //   ],
    //   gasPrice
    // );
  } catch (error) {
    l(error);
  }
}

main();
