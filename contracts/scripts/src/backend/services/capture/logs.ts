import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
  epochToDateString,
} from "../utils";

interface Log {
  blockTime: string;
  sender: string;
  counter: number;
}

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
  const name = NAME as "kujira" | "stargaze";

  const {
    oracle,
    adapter: { scheduler },
  } = await getCwQueryHelpers(chainId, RPC);

  const oracleLogs: Log[] = (await oracle.cwQueryLog(false)).map(
    ({ block_time, sender, counter }) => ({
      blockTime: `${block_time} (${epochToDateString(block_time)} UTC+3)`,
      sender,
      counter,
    })
  );
  const schedulerLogs: Log[] = (await scheduler[name].cwQueryLog(false)).map(
    ({ block_time, sender, counter }) => ({
      blockTime: `${block_time} (${epochToDateString(block_time)} UTC+3)`,
      sender,
      counter,
    })
  );

  // write files
  await writeFile(
    getSnapshotPath(NAME, TYPE, "main", "logs-oracle.json"),
    JSON.stringify(oracleLogs, null, 2),
    {
      encoding: ENCODING,
    }
  );

  await writeFile(
    getSnapshotPath(NAME, TYPE, "main", "logs-scheduler.json"),
    JSON.stringify(schedulerLogs, null, 2),
    {
      encoding: ENCODING,
    }
  );
}

main();
