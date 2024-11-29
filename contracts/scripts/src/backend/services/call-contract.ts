import { getSigner } from "../account/signer";
import { getLast, l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ADDRESS, TOKEN } from "../../common/config";
import { COLLECTION } from "./constants";
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
import { getFloorPrices } from "./sg-api";

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

    const LENDING_PLATFORM_CONTRACT = getContractByLabel(
      CONTRACTS,
      "lending_platform"
    );

    const ORACLE_CONTRACT = getContractByLabel(CONTRACTS, "oracle");

    // const ORACLE_CONTRACT = getContractByLabel(CONTRACTS, "oracle");
    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

    const sgQueryHelpers = await getSgQueryHelpers(RPC);
    const sgExecHelpers = await getSgExecHelpers(RPC, owner, signer);

    const { scheduler, lending, minter, oracle, utils, marketMaker } =
      await getCwQueryHelpers(chainId, RPC);
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const { getBalance, getAllBalances, getTimeInNanos } = sgQueryHelpers;
    const { sgMultiSend, sgIbcHookCall, sgSend } = sgExecHelpers;
    console.clear();

    // await getBalance(owner, TOKEN.NEUTRON.TESTNET.USDC);

    const collectionList = [
      COLLECTION.TESTNET?.PIGEON?.NEUTRON || "",
      COLLECTION.TESTNET?.BAD_KIDS?.NEUTRON || "",
      COLLECTION.TESTNET?.SLOTH?.NEUTRON || "",
    ];

    // // add liquidity to maker maker
    // for (const [collection] of collectionList) {
    //   const floorPrice =
    //     floorPrices.find((x) => x.collection === collection)?.price || 0;

    //   await h.marketMaker.cwDepositLiquidity(
    //     collection,
    //     floorPrice * 1e6 * 50,
    //     paymentDenom,
    //     gasPrice
    //   );
    // }

    // await h.minter.cwMintMultiple(
    //   TOKEN.NEUTRON.TESTNET.USDC,
    //   [[owner, 400_000_000_004]],
    //   gasPrice
    // );
    // await minter.cwQueryBalances(owner, true);

    // await h.marketMaker.cwDepositLiquidity(
    //   COLLECTION.TESTNET?.PIGEON?.NEUTRON || "",
    //   9 * 20 * 1e6,
    //   TOKEN.NEUTRON.TESTNET.USDC,
    //   gasPrice
    // );
    // await h.marketMaker.cwDepositLiquidity(
    //   COLLECTION.TESTNET?.BAD_KIDS?.NEUTRON || "",
    //   1_700 * 20 * 1e6,
    //   TOKEN.NEUTRON.TESTNET.USDC,
    //   gasPrice
    // );
    // await h.marketMaker.cwDepositLiquidity(
    //   COLLECTION.TESTNET?.SLOTH?.NEUTRON || "",
    //   6_800 * 20 * 1e6,
    //   TOKEN.NEUTRON.TESTNET.USDC,
    //   gasPrice
    // );

    // await oracle.pQueryPrices(9, 9, true);

    const collections = (await marketMaker.pQueryCollectionOwnerList(9)).map(
      (x) => x.collection_address
    );
    const liqudity = await marketMaker.pQueryLiquidityList(9);
    const prices = await marketMaker.pQueryOfferPricesList(9);

    li({ collections });
    li({ liqudity });
    li({ prices });

    return;

    await scheduler.cwQueryConfig(true);
    await lending.cwQueryAddressConfig(true);
    await lending.cwQueryCommonConfig(true);
    await lending.cwQueryRateConfig(true);
    await oracle.cwQueryConfig(true);
    await marketMaker.cwQueryConfig(true);
    return;

    // for (const collectionAddress of [
    //   COLLECTION.TESTNET?.PIGEON?.NEUTRON || "",
    //   COLLECTION.TESTNET?.BAD_KIDS?.NEUTRON || "",
    //   COLLECTION.TESTNET?.SLOTH?.NEUTRON || "",
    // ]) {
    //   await h.utils.cwMintNft(
    //     collectionAddress,
    //     ADDRESS.TESTNET.NEUTRON.WORKER,
    //     [4, 5, 6],
    //     gasPrice
    //   );
    // }

    return;

    // const res = await utils.cwV2QueryAsksByCollectionDenom(
    //   // "stars1fvhcnyddukcqfnt7nlwv3thm5we22lyxyxylr9h77cvgkcn43xfsvgv0pl" ||
    //   ADDRESS.MAINNET.STARGAZE.MARKETPLACE,
    //   COLLECTION.MAINNET.STARGAZE.EXPEDITION,
    //   TOKEN.MAINNET.STARS,
    //   { descending: false, limit: 5 }
    // );

    const res = await getFloorPrices([
      COLLECTION?.MAINNET?.EXPEDITION?.STARGAZE || "",
      COLLECTION?.MAINNET?.MAD_SCIENTISTS?.STARGAZE || "",
    ]);

    li(res);
  } catch (error) {
    l(error);
  }
}

main();
