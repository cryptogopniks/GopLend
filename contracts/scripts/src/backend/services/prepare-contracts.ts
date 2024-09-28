import { getSigner } from "../account/signer";
import { floor, getLast, l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { COLLECTION, TOKEN } from "./constants";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  getChainOptionById,
  getContractByLabel,
} from "../../common/config/config-utils";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
} from "./utils";

async function main() {
  try {
    const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
    const { chainId } = parseStoreArgs();
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

    const LENDING_PLATFORM_MAIN_CONTRACT = getContractByLabel(
      CONTRACTS,
      "lending_platform_main"
    );

    const LENDING_PLATFORM_STABLE_CONTRACT = getContractByLabel(
      CONTRACTS,
      "lending_platform_stable"
    );

    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

    const { minter, marketMaker } = await getCwQueryHelpers(chainId, RPC);
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const prepareConfigs = async () => {
      // setup minter
      const isMainnet = TYPE === "main";

      const bglDenomMain = "bglSTARS";
      const bglDenomStable = "bglUSDC";
      const paymentAmount = 10_000_000_000;
      const paymentDenomMain = isMainnet
        ? TOKEN.MAINNET.STARS
        : TOKEN.TESTNET.STARS;
      const paymentDenomStable = isMainnet
        ? TOKEN.MAINNET.STARS
        : TOKEN.TESTNET.STARS;
      const bglDecimals = 6;

      const platforms: [PlatformType, string, string, string][] = [
        [
          "main",
          LENDING_PLATFORM_MAIN_CONTRACT.ADDRESS,
          bglDenomMain,
          paymentDenomMain,
        ],
        [
          "stable",
          LENDING_PLATFORM_STABLE_CONTRACT.ADDRESS,
          bglDenomStable,
          paymentDenomStable,
        ],
      ];

      const collectionList = isMainnet
        ? [[COLLECTION.MAINNET.STARGAZE.PIGEON, "PIGEON"]]
        : [
            [COLLECTION.TESTNET.STARGAZE.MOO, "MOO"],
            [COLLECTION.TESTNET.STARGAZE.PIGEON, "PIGEON"],
            [COLLECTION.TESTNET.STARGAZE.SHANE, "SHANE"],

            [COLLECTION.TESTNET.STARGAZE.GOPNIK_GIRLS, "GOPNIK_GIRLS"],
            [COLLECTION.TESTNET.STARGAZE.PLASTELINE, "PLASTELINE"],
            [COLLECTION.TESTNET.STARGAZE.BIT_KIDS, "BIT_KIDS"],
            [COLLECTION.TESTNET.STARGAZE.BAD_KIDS, "BAD_KIDS"],
            [COLLECTION.TESTNET.STARGAZE.SLOTHS, "SLOTHS"],
          ];

      for (const [
        platformType,
        lendingPlatformContract,
        bglDenom,
        paymentDenom,
      ] of platforms) {
        await h.minter.stargaze.cwCreateNative(
          bglDenom,
          {
            decimals: bglDecimals,
            whitelist: [lendingPlatformContract],
          },
          paymentAmount,
          paymentDenom,
          gasPrice
        );

        const { currency: bglCurrency } = getLast(
          (await minter.stargaze.cwQueryCurrencyInfoListByOwner(owner)).filter(
            (x) => x.whitelist.includes(lendingPlatformContract)
          )
        );
        const bglDenomFull = (bglCurrency as any)?.token?.native?.denom || "";
        l({ bglDenomFull });

        // setup lending
        const isMarketplaceInterfaceEnabled =
          !isMainnet && platformType === "main";

        // borrowed = deposited + borrowers_reserve_fraction_ratio * reserve_pool
        // withdrawable = deposited + (borrow_apr - borrow_fee_rate) * time * borrowed
        // reserve_pool = borrowed + withdrawable

        // deposited = 100, reserve_pool = 1000, time = 3
        // borrowed = 100 + 0.45 * 1000 = 550
        // withdrawable = 100 + 0.2 * 3 * 550 = 430
        // reserve_pool = 550 + 430 = 980
        await h.lending.cwUpdateCommonConfig(
          platformType,
          {
            bglCurrency: {
              token: { native: { denom: bglDenomFull } },
              decimals: bglDecimals,
            },
            // collateralMinValue: 100,
            // unbondingPeriod: 60,
            // isMarketplaceInterfaceEnabled,
            // borrowersReserveFractionRatio: 0.45,
          },
          gasPrice
        );

        // // deposit reserve liquidity
        // await h.lending.cwDepositReserveLiquidity(
        //   platformType,
        //   1_000_000_000,
        //   { native: { denom: paymentDenom } },
        //   gasPrice
        // );

        // // add collections
        // let i = 0;

        // for (const [collection_address, name] of collectionList) {
        //   await h.lending.cwCreateProposal(
        //     platformType,
        //     {
        //       proposal_type: {
        //         add_collection: {
        //           collection: { name, owner },
        //           collection_address,
        //         },
        //       },
        //       listing_price: {
        //         amount: "1",
        //         currency: {
        //           token: { native: { denom: paymentDenom } },
        //           decimals: 6,
        //         },
        //       },
        //     },
        //     gasPrice
        //   );

        //   await h.lending.cwAcceptProposal(
        //     platformType,
        //     ++i,
        //     1,
        //     { native: { denom: paymentDenom } },
        //     gasPrice
        //   );
        // }
      }

      // add market maker outposts
      const hubConfig = await marketMaker.hub.cwQueryConfig();
      const maxOutpostAmount = hubConfig.max_outpost_amount;
      let outpostAmount = (await marketMaker.hub.cwQueryOutposts()).length;

      while (outpostAmount < maxOutpostAmount) {
        await h.marketMaker.hub.cwAddOutpost(gasPrice);
        outpostAmount = (await marketMaker.hub.cwQueryOutposts()).length;
      }

      // // set collection owners in market maker
      // const ownerList = hubConfig.worker
      //   ? [hubConfig.admin, hubConfig.worker]
      //   : [hubConfig.admin];

      // for (const [collection] of collectionList) {
      //   await h.marketMaker.hub.cwSetCollection(
      //     collection,
      //     ownerList,
      //     gasPrice
      //   );
      // }

      // // add liquidity to maker maker
      // for (const [collection] of collectionList) {
      //   await h.marketMaker.hub.cwDepositLiquidity(
      //     collection,
      //     floor(paymentAmount / maxOutpostAmount),
      //     paymentDenomMain,
      //     gasPrice
      //   );
      // }
    };

    await prepareConfigs();
  } catch (error) {
    l(error);
  }
}

main();
