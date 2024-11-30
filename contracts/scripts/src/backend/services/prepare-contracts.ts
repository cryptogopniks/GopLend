import { getSigner } from "../account/signer";
import { floor, getLast, l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { COLLECTION } from "./constants";
import { TOKEN } from "../../common/config";
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
import { getFloorPrices } from "./sg-api";

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

    const LENDING_PLATFORM_CONTRACT = getContractByLabel(
      CONTRACTS,
      "lending_platform"
    );

    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
    const testWallets = await getWallets(TYPE);
    const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

    const { minter, marketMaker } = await getCwQueryHelpers(chainId, RPC);
    const h = await getCwExecHelpers(chainId, RPC, owner, signer);

    const prepareConfigs = async () => {
      // setup minter
      const isMainnet = TYPE === "main";

      const bglDenom = "bglUSDC";
      const paymentAmount = 1;
      const paymentDenom = isMainnet
        ? TOKEN.NEUTRON.MAINNET.USDC
        : TOKEN.NEUTRON.TESTNET.USDC;
      const bglDecimals = 6;

      const collectionEntries = isMainnet
        ? Object.entries(COLLECTION?.MAINNET || {})
        : Object.entries(COLLECTION?.TESTNET || {});

      const collectionList = collectionEntries
        .map(([name, networkItem]) => {
          const address = networkItem?.NEUTRON || "";
          return [address, name];
        })
        .filter(([address]) => address) as [string, string][];

      const addressList = collectionList.map(([address]) => address);
      const floorPrices = await getFloorPrices(addressList);

      await h.minter.cwCreateNative(
        bglDenom,
        {
          decimals: bglDecimals,
          whitelist: [LENDING_PLATFORM_CONTRACT.ADDRESS],
        },
        paymentAmount,
        paymentDenom,
        gasPrice
      );

      const { currency: bglCurrency } = getLast(
        (await minter.cwQueryCurrencyInfoListByOwner(owner)).filter((x) =>
          x.whitelist.includes(LENDING_PLATFORM_CONTRACT.ADDRESS)
        )
      );
      const bglDenomFull = (bglCurrency as any)?.token?.native?.denom || "";
      l({ bglDenomFull });
      return;

      // setup lending

      // borrowed = deposited + borrowers_reserve_fraction_ratio * reserve_pool
      // withdrawable = deposited + (borrow_apr - borrow_fee_rate) * time * borrowed
      // reserve_pool = borrowed + withdrawable

      // deposited = 100, reserve_pool = 1000, time = 3
      // borrowed = 100 + 0.45 * 1000 = 550
      // withdrawable = 100 + 0.2 * 3 * 550 = 430
      // reserve_pool = 550 + 430 = 980
      await h.lending.cwUpdateCommonConfig(
        {
          bglCurrency: {
            token: { native: { denom: bglDenomFull } },
            decimals: bglDecimals,
          },
          collateralMinValue: 100,
          unbondingPeriod: 60,
          borrowersReserveFractionRatio: 0.45,
        },
        gasPrice
      );
      return;

      // deposit reserve liquidity
      await h.lending.cwDepositReserveLiquidity(
        1_000_000_000,
        { native: { denom: paymentDenom } },
        gasPrice
      );

      // add collections
      let i = 0;

      for (const [collection_address, name] of collectionList) {
        await h.lending.cwCreateProposal(
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
          ++i,
          1,
          { native: { denom: paymentDenom } },
          gasPrice
        );
      }

      // add market maker outposts
      const hubConfig = await marketMaker.cwQueryConfig();

      // set collection owner in market maker
      for (const [collection] of collectionList) {
        await h.marketMaker.cwSetCollection(
          collection,
          hubConfig.admin,
          gasPrice
        );
      }

      // add liquidity to maker maker
      for (const [collection] of collectionList) {
        const floorPrice =
          floorPrices.find((x) => x.collection === collection)?.price || 0;

        await h.marketMaker.cwDepositLiquidity(
          collection,
          floorPrice * 1e6 * 50,
          paymentDenom,
          gasPrice
        );
      }
    };

    await prepareConfigs();
  } catch (error) {
    l(error);
  }
}

main();
