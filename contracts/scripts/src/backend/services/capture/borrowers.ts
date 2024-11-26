import { l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
} from "../utils";

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

  const writeBorrowers = async () => {
    try {
      // sort by (loan + accumulated_loan) descending
      const borrowers = (
        await lending.pQueryBorrowerList(PAGINATION_QUERY_AMOUNT)
      ).sort(
        (a, b) =>
          Number(b.borrower.loan.amount) +
          Number(b.borrower.accumulated_loan) -
          (Number(a.borrower.loan.amount) + Number(a.borrower.accumulated_loan))
      );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "borrowers.json"),
        JSON.stringify(borrowers, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeBorrowers();
}

main();
