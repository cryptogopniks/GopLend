import { getSigner } from "../account/signer";
import { l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ADDRESS } from "../../common/config";
import { COLLECTION, TOKEN } from "./constants";
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

async function main() {
  try {
    const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
    const { chainId } = parseStoreArgs();
    const {
      NAME,
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

    const { adapter, lending, minter, oracle, utils } = await getCwQueryHelpers(
      chainId,
      RPC
    );
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const { getAllBalances } = sgQueryHelpers;
    const { sgMultiSend } = sgExecHelpers;

    const name = NAME as "kujira" | "stargaze";

    // await adapter.marketplace.stargaze.cwQueryConfig();
    // await adapter.scheduler[name].cwQueryConfig();
    // await lending.cwQueryAddressConfig();
    // await lending.cwQueryRateConfig();
    // await lending.cwQueryCommonConfig();
    // await minter[name].cwQueryConfig();
    // await oracle.cwQueryConfig();

    // await lending.cwQueryCollectionList();

    // await h.minter.kujira.cwCreateNative(
    //   owner,
    //   "test",
    //   6,
    //   100_000_000,
    //   "ukuji",
    //   gasPrice
    // );
    // await minterKujira.cwQueryCurrenciesByCreator(owner);

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
          collection_address: COLLECTION.KUJIRA.BANDIT_2,
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
