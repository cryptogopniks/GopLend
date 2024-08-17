import { getSigner } from "../account/signer";
import { l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { getChainOptionById } from "../../common/config/config-utils";
import { queryTargets } from "./query-targets";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { Route } from "../../common/codegen/AdapterDexStargaze.types";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";
import {
  LiquidationItem,
  QueryBorrowersResponseItem,
} from "../../common/codegen/LendingPlatform.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
} from "./utils";

const PAGINATION_QUERY_AMOUNT = 100;
const PAGINATION_EXECUTE_AMOUNT = 3;

async function liquidate(
  route: Route,
  queryBorrowerListFn: (
    platformType: PlatformType,
    paginationQueryAmount: number
  ) => Promise<QueryBorrowersResponseItem[]>,
  pushFn: (
    route: Route,
    liquidationItemList: LiquidationItem[]
  ) => Promise<void>
) {
  try {
    const platformType: PlatformType = route === "empty" ? "main" : "stable";

    // query all borrowers
    const borrowerAddressList = (
      await queryBorrowerListFn(platformType, PAGINATION_QUERY_AMOUNT)
    ).map((x) => x.address);

    // get targets
    const targets = await queryTargets(platformType, borrowerAddressList);

    // liquidate with pagination
    let liquidationItemList: LiquidationItem[] = [];

    for (let i = 0; i < targets.length; i++) {
      if (liquidationItemList.length >= PAGINATION_EXECUTE_AMOUNT) {
        l(i);
        await pushFn(route, liquidationItemList);
        liquidationItemList = [];
      }

      liquidationItemList.push(targets[i]);
    }
  } catch (error) {
    l(error);
  }
}

async function main() {
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
      GAS_PRICE_AMOUNT,
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
  const testWallets = await getWallets(TYPE);
  const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

  const { lending } = await getCwQueryHelpers(chainId, RPC);
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);

  const name = NAME as "kujira" | "stargaze";

  const cwQueryBorrowerList = async (
    platformType: PlatformType,
    paginationQueryAmount: number
  ): Promise<QueryBorrowersResponseItem[]> => {
    return await lending.pQueryBorrowerList(
      platformType,
      paginationQueryAmount,
      0,
      true
    );
  };

  const cwPush = async (
    route: Route,
    liquidationItemList: LiquidationItem[]
  ) => {
    await h.adapter.scheduler[name].cwPush(
      route,
      liquidationItemList,
      gasPrice
    );
  };

  let route: Route = "empty";

  // run liquidation in infinite loop
  while (true) {
    await liquidate(route, cwQueryBorrowerList, cwPush);

    // switch route
    if (route === "empty") {
      // TODO: query network/dex status and choose route
      route = "osmosis";
    } else {
      route = "empty";
    }
  }
}

main();
