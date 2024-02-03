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

async function main() {
  try {
    const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
    const CHAIN_ID_STARGAZE = "stargaze-0";

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
  } catch (error) {
    l(error);
  }
}

main();
