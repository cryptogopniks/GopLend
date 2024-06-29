import { getSigner } from "../account/signer";
import { getLast, l, li, wait } from "../../common/utils";
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
const PAGINATION_EXECUTE_AMOUNT = 10;

async function liquidate(
  platformType: PlatformType,
  route: Route,
  queryBorrowerListFn: (
    platformType: PlatformType,
    paginationQueryAmount: number,
    lastItem?: string
  ) => Promise<QueryBorrowersResponseItem[]>,
  pushFn: (
    route: Route,
    liquidationItemList: LiquidationItem[]
  ) => Promise<void>
) {
  try {
    // query all borrowers
    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryBorrowersResponseItem[] = await queryBorrowerListFn(
        platformType,
        PAGINATION_QUERY_AMOUNT,
        lastItem
      );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastBorrower: lastItem });
    }

    const borrowerAddressList = allItems.map((x) => x.address);

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
    PREFIX: PREFIX,
    OPTION: {
      RPC_LIST: [RPC],
      DENOM: DENOM,
      GAS_PRICE_AMOUNT: GAS_PRICE_AMOUNT,
      TYPE: TYPE,
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
    paginationQueryAmount: number,
    lastItem?: string
  ): Promise<QueryBorrowersResponseItem[]> => {
    return await lending.cwQueryBorrowerList(
      platformType,
      paginationQueryAmount,
      lastItem
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

  let platform: PlatformType = "main";
  let route: Route = "empty";

  // run liquidation in infinite loop
  while (true) {
    await liquidate(platform, route, cwQueryBorrowerList, cwPush);

    // switch platform and route
    if (platform === "main") {
      platform = "stable";
      // TODO: query network/dex status and choose route
      route = "neutron" || "osmosis";
    } else {
      platform = "main";
      route = "empty";
    }
  }
}

main();
