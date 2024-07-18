import { getSigner } from "../account/signer";
import { l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ADDRESS } from "../../common/config";
import { COLLECTION, TOKEN } from "./constants";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
} from "./utils";
import {
  getChainOptionById,
  getContractByLabel,
} from "../../common/config/config-utils";
import {
  getSgQueryHelpers,
  getSgExecHelpers,
} from "../../common/account/sg-helpers";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";

async function main() {
  try {
    const { chainId } = parseStoreArgs();
    const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
    const {
      NAME,
      PREFIX,
      OPTION: {
        RPC_LIST: [RPC],
        DENOM,
        CONTRACTS,
        GAS_PRICE_AMOUNT,
        TYPE,
      },
    } = getChainOptionById(CHAIN_CONFIG, chainId);

    // const LENDING_PLATFORM_CONTRACT = getContractByWasm(
    //   CONTRACTS,
    //   "lending_platform.wasm"
    // );
    // const ORACLE_CONTRACT = getContractByWasm(CONTRACTS, "oracle.wasm");
    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

    const sgQueryHelpers = await getSgQueryHelpers(RPC);
    const sgExecHelpers = await getSgExecHelpers(RPC, owner, signer);

    const { adapter, lending, minter, oracle, utils, faucet } =
      await getCwQueryHelpers(chainId, RPC);
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const { getBalance, getAllBalances, getTimeInNanos } = sgQueryHelpers;
    const { sgMultiSend, sgIbcHookCall } = sgExecHelpers;

    const name = NAME as "kujira" | "stargaze";

    await lending.cwQueryCommonConfig("main", true);
    return;

    await lending.cwQueryBalances("main", true);
    await lending.cwQueryConditionalDepositApr("main", 0, true);
    return;

    const CURRENCY = {
      STARS: {
        token: { native: { denom: TOKEN.STARS } },
        decimals: 6,
      },
      USDC: {
        token: { native: { denom: TOKEN.USDC } },
        decimals: 6,
      },
    };

    await h.oracle.cwUpdatePrices(
      0.0097,
      [
        {
          collection_address: COLLECTION.STARGAZE.PIGEON,
          price: {
            amount: "20000000",
            // amount: "30000000",
            currency: CURRENCY.STARS,
          },
          usd_price: {
            amount: "194000",
            // amount: "291000",
            currency: CURRENCY.USDC,
          },
        },
        {
          collection_address: COLLECTION.STARGAZE.SHANE,
          price: {
            amount: "10000000",
            currency: CURRENCY.STARS,
          },
          usd_price: {
            amount: "97000",
            currency: CURRENCY.USDC,
          },
        },
      ],
      gasPrice
    );
    // await oracle.cwQueryPrices(100, undefined, undefined, true);
    // await oracle.cwQueryPriceRatio(true);

    // const amountToBorrow = await lending.cwQueryAvailableToBorrow(
    //   "main",
    //   owner,
    //   undefined,
    //   true
    // );
    // await h.lending.cwBorrow(
    //   "main",
    //   Math.floor(0.9 * Number(amountToBorrow)),
    //   gasPrice
    // );

    // await lending.cwQueryConditionalLtv(
    //   "main",
    //   owner,
    //   undefined,
    //   undefined,
    //   true
    // );

    return;

    // const CHAID_ID_TERRA = "terra-0";
    // const DEX_TERRA =
    //   "terra14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9ssrc8au";
    // const [RPC_TERRA] = getChainOptionById(CHAIN_CONFIG, CHAID_ID_TERRA).OPTION
    //   .RPC_LIST;

    // await (
    //   await getCwQueryHelpers(CHAID_ID_TERRA, RPC_TERRA)
    // ).adapter.dex.stargaze.cwQueryConfig(true);
    // await (await getSgQueryHelpers(RPC_TERRA)).getAllBalances(DEX_TERRA);

    // await h.adapter.dex.stargaze.cwSwap(
    //   10_000_000,
    //   { native: { denom: "ustars" } },
    //   "test2",
    //   0.009,
    //   gasPrice
    // );

    // await wait(10_000);
    // await (
    //   await getCwQueryHelpers(CHAID_ID_TERRA, RPC_TERRA)
    // ).adapter.dex.stargaze.cwQueryConfig(true);
    // await (await getSgQueryHelpers(RPC_TERRA)).getAllBalances(DEX_TERRA);
    // return;

    await h.adapter.dex.stargaze.cwSwap(
      10_000_000,
      { native: { denom: "ustars" } },
      "osmosis",
      0.009,
      gasPrice
    );
    return;

    // const p =
    //   '{"wasm":{"contract":"osmo1vkdakqqg5htq5c3wy2kj2geq536q665xdexrtjuwqckpads2c2nsvhhcyv","msg":{"swap_and_action":{"user_swap":{"swap_exact_asset_in":{"swap_venue_name":"osmosis-poolmanager","operations":[{"pool":"1048","denom_in":"ibc/126DA09104B71B164883842B769C0E9EC1486C0887D27A9999E395C2C8FB5682","denom_out":"ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858"},{"pool":"1212","denom_in":"ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858","denom_out":"ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4"}]}},"min_asset":{"native":{"denom":"ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4","amount":"42783"}},"timeout_timestamp":1719426656827535276,"post_swap_action":{"transfer":{"to_address":"osmo1j5ft99lyd36e5fyp8kh8ze7qcj00relm0md4k9"}},"affiliates":[]}}}}';

    // li(JSON.parse(p));
    // return;

    // const timeInNanos = await getTimeInNanos();

    // await h.faucet.stargaze.cwClaimFor([[owner, 20_000_000_000]], gasPrice);
    // await getBalance(owner, TOKEN.USDC);
    // return;

    // // ntrn -> osmo
    // // https://osmosis-api.lavenderfive.com/cosmos/bank/v1beta1/balances/osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw
    // const res = await sgIbcHookCall(
    //   "osmo1vkdakqqg5htq5c3wy2kj2geq536q665xdexrtjuwqckpads2c2nsvhhcyv",
    //   0.01 * 1e6,
    //   "untrn",
    //   {
    //     swap_and_action: {
    //       user_swap: {
    //         swap_exact_asset_in: {
    //           swap_venue_name: "osmosis-poolmanager",
    //           operations: [
    //             {
    //               pool: "1388",
    //               denom_in:
    //                 "ibc/126DA09104B71B164883842B769C0E9EC1486C0887D27A9999E395C2C8FB5682",
    //               denom_out: "uosmo",
    //             },
    //           ],
    //         },
    //       },
    //       min_asset: {
    //         native: {
    //           denom: "uosmo",
    //           amount: "100",
    //         },
    //       },
    //       timeout_timestamp: timeInNanos + 10 * 60 * 1e9,
    //       post_swap_action: {
    //         transfer: {
    //           to_address:
    //             "osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw",
    //         },
    //       },
    //       affiliates: [],
    //     },
    //   },
    //   "transfer",
    //   "channel-10",
    //   timeInNanos,
    //   gasPrice
    // );

    // li(res);
    // return;

    // await h.adapter.dex.stargaze.cwSwap(
    //   310_000,
    //   { native: { denom: "untrn" } },
    //   "test",
    //   0.764,
    //   gasPrice
    // );
    // return;

    // await adapter.marketplace.stargaze.cwQueryConfig(true);
    // await adapter.scheduler[name].cwQueryConfig(true);
    // const platforms: PlatformType[] = ["main", "stable"];
    // for (const platformType of platforms) {
    //   await lending.cwQueryAddressConfig(platformType, true);
    //   await lending.cwQueryRateConfig(platformType, true);
    //   await lending.cwQueryCommonConfig(platformType, true);
    // }
    // await minter[name].cwQueryConfig(true);
    // await oracle.cwQueryConfig(true);
    // return;

    // await lending.cwQueryCollectionList(true);

    // await h.minter.kujira.cwCreateNative(
    //   owner,
    //   "test",
    //   6,
    //   100_000_000,
    //   "ukuji",
    //   gasPrice
    // );
    // await minterKujira.cwQueryCurrenciesByCreator(owner, true);

    // await h.minter.kujira.cwMint(
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
    // await lending.cwQueryUnbonder(owner, true);

    // await wait(60_000);
    // await h.lending.cwWithdraw(gasPrice);
    // await getAllBalances(owner);

    // await minterKujira.cwQueryCurrenciesByCreator(
    //   LENDING_PLATFORM_CONTRACT.ADDRESS, true
    // );

    // await updateConfigs();

    // await utils.cwQueryBalanceInNft(owner, COLLECTION.BANDIT_3, true); // 2,3 - [ '5', '6', '7', '8' ]
    // await utils.cwQueryOperators(owner, COLLECTION.BANDIT_1, true);

    // await lending.cwQueryCollectionList(true);

    await lending.cwQueryCollateralByOwner("main", owner, true);

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
      "main",
      [
        {
          collection_address: COLLECTION.KUJIRA.BANDIT_2,
          token_id_list: ["5"],
        },
      ],
      gasPrice
    );

    await lending.cwQueryCollateralByOwner("main", owner, true);

    // await oracle.cwQueryPrices(true);

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
