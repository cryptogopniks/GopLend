import { getSigner } from "../account/signer";
import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ENCODING, PATH_TO_CONFIG_JSON, getWallets } from "./utils";
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
    const CHAIN_ID_STARGAZE = "elgafar-1";

    const {
      PREFIX: PREFIX_STARGAZE,
      OPTION: {
        RPC_LIST: [RPC_STARGAZE],
        DENOM: DENOM_STARGAZE,
        CONTRACTS: CONTRACTS_STARGAZE,
        GAS_PRICE_AMOUNT: GAS_PRICE_AMOUNT_STARGAZE,
        TYPE: TYPE_STARGAZE,
      },
    } = getChainOptionById(CHAIN_CONFIG, CHAIN_ID_STARGAZE);

    const LENDING_PLATFORM_CONTRACT = getContractByWasm(
      CONTRACTS_STARGAZE,
      "lending_platform.wasm"
    );
    const ORACLE_CONTRACT = getContractByWasm(
      CONTRACTS_STARGAZE,
      "oracle.wasm"
    );
    const gasPriceStargaze = `${GAS_PRICE_AMOUNT_STARGAZE}${DENOM_STARGAZE}`;
    const testWallets = await getWallets(TYPE_STARGAZE);
    const { signer: signerStargaze, owner: ownerStargaze } = await getSigner(
      PREFIX_STARGAZE,
      testWallets.SEED_ADMIN
    );

    const sgQueryHelpersStargaze = await getSgQueryHelpers(RPC_STARGAZE);
    const sgExecHelpersStargaze = await getSgExecHelpers(
      RPC_STARGAZE,
      ownerStargaze,
      signerStargaze
    );

    const { cwQueryLendingPlatformConfig, cwQueryOracleConfig, cwQueryPrices } =
      await getCwQueryHelpers(CHAIN_ID_STARGAZE, RPC_STARGAZE);
    const { cwUpdateConfig, cwUpdatePrices } = await getCwExecHelpers(
      CHAIN_ID_STARGAZE,
      RPC_STARGAZE,
      ownerStargaze,
      signerStargaze
    );

    // await cwQueryLendingPlatformConfig();
    // await cwQueryOracleConfig();

    await cwUpdatePrices(
      [
        {
          collection_address:
            "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw",
          currency: { token: { native: { denom: "ustars" } }, decimals: 6 },
          raw_prices: ["100", "200", "300", "400", "500"],
          trading_volume_per_day: "1000",
        },
      ],
      gasPriceStargaze
    );

    await cwQueryPrices();
  } catch (error) {
    l(error);
  }
}

main();
