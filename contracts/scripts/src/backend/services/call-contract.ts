import { getSigner } from "../account/signer";
import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
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

    const { cwQueryLendingPlatformConfig, cwQueryOracleConfig, cwQueryPrices } =
      await getCwQueryHelpers(chainId, RPC);
    const { cwUpdateConfig, cwUpdatePrices } = await getCwExecHelpers(
      chainId,
      RPC,
      owner,
      signer
    );

    await cwQueryLendingPlatformConfig();
    // await cwQueryOracleConfig();

    // await cwUpdatePrices(
    //   [
    //     {
    //       collection_address:
    //         "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw",
    //       currency: { token: { native: { denom: "ustars" } }, decimals: 6 },
    //       raw_prices: ["100", "200", "300", "400", "500"],
    //       trading_volume_per_day: "1000",
    //     },
    //   ],
    //   gasPrice
    // );

    // await cwQueryPrices();
  } catch (error) {
    l(error);
  }
}

main();
