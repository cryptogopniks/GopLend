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

    const updateConfigs = async () => {
      await h.minter.cwCreateNative(
        LENDING_PLATFORM_CONTRACT.ADDRESS,
        "bglKUJI",
        6,
        1,
        "ukuji",
        gasPrice
      );

      await h.lending.cwUpdateCommonConfig(
        {
          collateralMinValue: 1_000,
          unbondingPeriod: 60,
          priceUpdatePeriod: 15 * 60,
        },
        gasPrice
      );
    };

    const collectionAddress =
      "kujira1lnafxjcagzt4qlztarxskpgrlny9d936vmvsx4m4qv0mengwlyes5q84p0";

    await updateConfigs();

    // await utils.cwQueryBalanceInNft(owner, collectionAddress);
    // await utils.cwQueryOperators(owner, collectionAddress);

    // await lending.cwQueryCollectionList();

    // await h.lending.cwApproveAndDepositCollateral(
    //   owner,
    //   LENDING_PLATFORM_CONTRACT.ADDRESS,
    //   [
    //     {
    //       collection_address: collectionAddress,
    //       token_id_list: ["5"],
    //     },
    //   ],
    //   gasPrice
    // );

    await lending.cwQueryAddressConfig();
    await lending.cwQueryRateConfig();
    await lending.cwQueryCommonConfig();

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
