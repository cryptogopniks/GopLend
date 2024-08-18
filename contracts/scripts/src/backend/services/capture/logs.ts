import { l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import { Route } from "../../../common/codegen/AdapterDexStargaze.types";
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
  route?: Route; // scheduler
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
  const name = NAME as "kujira" | "stargaze";

  const {
    oracle,
    adapter: { scheduler },
  } = await getCwQueryHelpers(chainId, RPC);

  const writeOracleLogs = async () => {
    try {
      const oracleLogs: Log[] = (await oracle.cwQueryLog(false)).map(
        ({ block_time, sender, outdated_price_collections, counter }) => ({
          blockTime: getBlockTime(block_time, TIMEZONE),
          sender,
          outdatedPriceCollections: outdated_price_collections,
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
    } catch (error) {
      l(error);
    }
  };

  const writeSchedulerLogs = async () => {
    try {
      const schedulerLogs: Log[] = (
        await scheduler[name].cwQueryLog(false)
      ).map(({ block_time, sender, route, counter }) => ({
        blockTime: getBlockTime(block_time, TIMEZONE),
        sender,
        route,
        counter,
      }));

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "main", "logs-scheduler.json"),
        JSON.stringify(schedulerLogs, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await Promise.all([writeOracleLogs(), writeSchedulerLogs()]);
}

main();
