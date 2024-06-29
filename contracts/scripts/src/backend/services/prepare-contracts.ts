import { getSigner } from "../account/signer";
import { l } from "../../common/utils";
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
      PREFIX: PREFIX,
      OPTION: {
        RPC_LIST: [RPC],
        DENOM: DENOM,
        CONTRACTS: CONTRACTS,
        GAS_PRICE_AMOUNT: GAS_PRICE_AMOUNT,
        TYPE: TYPE,
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

    const { minter } = await getCwQueryHelpers(chainId, RPC);
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const prepareConfigs = async () => {
      // setup minter
      const name = NAME as "kujira" | "stargaze";
      const isKujira = name === "kujira";

      const bglDenomMain = isKujira ? "bglKUJI" : "bglSTARS";
      const bglDenomStable = isKujira ? "bglUSK" : "bglUSDC";
      const paymentAmount = isKujira ? 100_000_000 : 10_000_000_000;
      const paymentDenomMain = isKujira ? TOKEN.KUJI : TOKEN.STARS;
      const paymentDenomStable = isKujira ? TOKEN.USK : TOKEN.USDC;
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

      for (const [
        platformType,
        lendingPlatformContract,
        bglDenom,
        paymentDenom,
      ] of platforms) {
        await h.minter[name].cwCreateNative(
          lendingPlatformContract,
          bglDenom,
          bglDecimals,
          paymentAmount,
          paymentDenom,
          gasPrice
        );

        const [bglCurrency] = (
          await minter[name].cwQueryCurrenciesByCreator(lendingPlatformContract)
        ).currencies;

        const bglDenomFull = (bglCurrency as any)?.token?.native?.denom || "";
        l({ bglDenomFull });

        // setup lending
        const isMarketplaceInterfaceEnabled =
          !isKujira && platformType === "main";

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
            collateralMinValue: 100,
            unbondingPeriod: 60,
            isMarketplaceInterfaceEnabled,
            borrowersReserveFractionRatio: 0.45,
          },
          gasPrice
        );

        // deposit reserve liquidity
        await h.lending.cwDepositReserveLiquidity(
          platformType,
          1_000_000_000,
          { native: { denom: paymentDenom } },
          gasPrice
        );

        // add collections
        const collections = isKujira
          ? [
              [COLLECTION.KUJIRA.BANDIT_1, "BANDIT_1"],
              [COLLECTION.KUJIRA.BANDIT_2, "BANDIT_2"],
              [COLLECTION.KUJIRA.BANDIT_3, "BANDIT_3"],
            ]
          : [
              [COLLECTION.STARGAZE.PIGEON, "PIGEON"],
              [COLLECTION.STARGAZE.SHANE, "SHANE"],
            ];

        let i = 0;

        for (const [collection_address, name] of collections) {
          await h.lending.cwCreateProposal(
            platformType,
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
                  token: { native: { denom: paymentDenom } },
                  decimals: 6,
                },
              },
            },
            gasPrice
          );

          await h.lending.cwAcceptProposal(
            platformType,
            ++i,
            1,
            { native: { denom: paymentDenom } },
            gasPrice
          );
        }
      }
    };

    await prepareConfigs();
  } catch (error) {
    l(error);
  }
}

main();
