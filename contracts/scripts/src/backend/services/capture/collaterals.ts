import { l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { Addr, TokenItem } from "../../../common/codegen/LendingPlatform.types";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
} from "../utils";

interface CollateralByOwner {
  collection: Addr;
  token_item_list: TokenItem[];
}

interface CollateralsByOwnerListItem {
  owner: Addr;
  collateral: CollateralByOwner[];
}

const PAGINATION_QUERY_AMOUNT = 200;

async function main() {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    NAME,
    OPTION: {
      RPC_LIST: [RPC],
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { lending } = await getCwQueryHelpers(chainId, RPC);

  const writeCollaterals = async () => {
    try {
      let collateralList = await lending.pQueryCollateralList(
        PAGINATION_QUERY_AMOUNT
      );

      // get list of collateral by owner
      const ownerList = Array(
        ...new Set(
          collateralList.map((x) => x.collateral.map((y) => y.owner)).flat()
        )
      );

      let collateralsByOwnerList: CollateralsByOwnerListItem[] = [];

      for (const currentOwner of ownerList) {
        let collateralByOwner: CollateralByOwner[] = [];

        for (const {
          address: collectionAddress,
          collateral,
        } of collateralList) {
          for (const { owner, token_item_list } of collateral) {
            if (currentOwner === owner) {
              collateralByOwner.push({
                collection: collectionAddress,
                token_item_list,
              });
            }
          }
        }

        collateralsByOwnerList.push({
          owner: currentOwner,
          collateral: collateralByOwner,
        });
      }

      // sort by tokens amount descending
      collateralsByOwnerList = collateralsByOwnerList.sort(
        (a, b) =>
          b.collateral.reduce(
            (acc, cur) => acc + cur.token_item_list.length,
            0
          ) -
          a.collateral.reduce((acc, cur) => acc + cur.token_item_list.length, 0)
      );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "collaterals.json"),
        JSON.stringify(collateralsByOwnerList, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeCollaterals();
}

main();
