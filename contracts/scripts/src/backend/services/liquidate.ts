import { getSigner } from "../account/signer";
import { getLast, l, li, wait } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { getChainOptionById } from "../../common/config/config-utils";
import { queryTargets } from "./query-targets";
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
  queryBorrowerListFn: (
    paginationQueryAmount: number,
    lastItem?: string
  ) => Promise<QueryBorrowersResponseItem[]>,
  pushFn: (liquidationItemList: LiquidationItem[]) => Promise<void>
) {
  try {
    // query all borrowers
    let allItems: QueryBorrowersResponseItem[] = [];
    let lastItem: string | undefined = undefined;

    while (lastItem !== "") {
      const items: QueryBorrowersResponseItem[] = await queryBorrowerListFn(
        PAGINATION_QUERY_AMOUNT,
        lastItem
      );

      lastItem = getLast(items)?.address || "";
      allItems = [...allItems, ...items];

      l({ lastBorrower: lastItem });
    }

    const borrowerAddressList = allItems.map((x) => x.address);

    // get targets
    const targets = await queryTargets(borrowerAddressList);

    // liquidate with pagination
    let liquidationItemList: LiquidationItem[] = [];

    for (let i = 0; i < targets.length; i++) {
      if (liquidationItemList.length >= PAGINATION_EXECUTE_AMOUNT) {
        l(i);
        await pushFn(liquidationItemList);
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
    paginationQueryAmount: number,
    lastItem?: string
  ): Promise<QueryBorrowersResponseItem[]> => {
    return await lending.cwQueryBorrowerList(paginationQueryAmount, lastItem);
  };

  const cwPush = async (liquidationItemList: LiquidationItem[]) => {
    await h.adapter.scheduler[name].cwPush(liquidationItemList, gasPrice);
  };

  // run liquidation in infinite loop
  while (true) {
    await liquidate(cwQueryBorrowerList, cwPush);
  }
}

main();
