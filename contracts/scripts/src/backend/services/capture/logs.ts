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
  epochToDateString,
  epochToDateStringUTC,
} from "../utils";

interface Log {
  blockTime: string;
  sender: string;
  outdatedPriceCollections?: string[]; // oracle
  counter: number;
}

type Timezone = "UTC" | "UTC+3";

const TIMEZONE: Timezone = "UTC+3";

function getBlockTime(blockTime: number, timezone: Timezone) {
  const dateString =
    timezone === "UTC"
      ? epochToDateStringUTC(blockTime)
      : epochToDateString(blockTime);
  return `${blockTime} (${dateString} ${timezone})`;
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

  const { scheduler } = await getCwQueryHelpers(chainId, RPC);

  const writeSchedulerLogs = async () => {
    try {
      const schedulerLogs: Log[] = (await scheduler.cwQueryLog(false)).map(
        ({ block_time, sender, counter }) => ({
          blockTime: getBlockTime(block_time, TIMEZONE),
          sender,
          counter,
        })
      );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "logs-scheduler.json"),
        JSON.stringify(schedulerLogs, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeSchedulerLogs();
}

main();
