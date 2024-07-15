import { getSigner } from "../account/signer";
import { getLast, l, li } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { getChainOptionById } from "../../common/config/config-utils";
import { queryTargets } from "./query-targets";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import { QueryBorrowersResponseItem } from "../../common/codegen/LendingPlatform.types";
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

const PAGINATION_QUERY_AMOUNT = 100;

async function main(platformType: PlatformType) {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    PREFIX,
    OPTION: {
      RPC_LIST: [RPC],
      DENOM,
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const testWallets = await getWallets(TYPE);
  const { signer, owner } = await getSigner(PREFIX, testWallets.SEED_ADMIN);

  const { lending } = await getCwQueryHelpers(chainId, RPC);
  const h = await getCwExecHelpers(chainId, RPC, owner, signer);

  // query all borrowers
  let allItems: QueryBorrowersResponseItem[] = [];
  let lastItem: string | undefined = undefined;

  while (lastItem !== "") {
    const items: QueryBorrowersResponseItem[] =
      await lending.cwQueryBorrowerList(
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
  li({ targets });
}

main("main");
