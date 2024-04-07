import { getSigner } from "../account/signer";
import { l } from "../../common/utils";
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
    const { signer: signer, owner: owner } = await getSigner(
      PREFIX,
      testWallets.SEED_ADMIN
    );

    const sgQueryHelpers = await getSgQueryHelpers(RPC);
    const sgExecHelpers = await getSgExecHelpers(RPC, owner, signer);

    const { adapter, lending, minter, oracle } = await getCwQueryHelpers(
      chainId,
      RPC
    );
    // const { cwUpdateConfig, cwUpdatePrices } = await getCwExecHelpers(
    //   chainId,
    //   RPC,
    //   owner,
    //   signer
    // );

    const { getAllBalances } = sgQueryHelpers;
    const { sgMultiSend } = sgExecHelpers;

    // await lending.cwQueryAddressConfig();
    // await lending.cwQueryRateConfig();
    // await lending.cwQueryCommonConfig();

    await oracle.cwQueryPrices();

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
