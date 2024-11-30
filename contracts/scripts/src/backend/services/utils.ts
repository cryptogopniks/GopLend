import { access, readFile } from "fs/promises";
import { decrypt, floor, getLast, l } from "../../common/utils";
import { PATH, rootPath } from "../envs";
import { Label } from "../../common/config";
import { ChainType, Wallets, StoreArgs } from "../../common/interfaces";
import { NetworkItem } from "./constants";

const ENCODING = "utf8";
const PATH_TO_CONFIG_JSON = rootPath("./src/common/config/config.json");

// "$CHAIN_ID|$LABEL_A,$LABEL_B"
function parseStoreArgs(): StoreArgs {
  const args = getLast(process.argv).trim();
  if (args.includes("/")) throw new Error("Store args are not specified!");

  const [chainId, labelListString] = args.split("|");
  const labelList = labelListString.split(",").map((x) => x as Label);

  return {
    chainId,
    labelList,
  };
}

function parseChainId(): string {
  const arg = getLast(process.argv).trim();
  if (arg.includes("/")) throw new Error("Network name is not specified!");

  return arg;
}

async function decryptSeed(seedEncrypted: string) {
  const keyPath = rootPath(PATH.TO_ENCRYPTION_KEY);

  await access(keyPath);
  const encryptionKey = await readFile(keyPath, { encoding: ENCODING });
  const seed = decrypt(seedEncrypted, encryptionKey);
  if (!seed) throw new Error("The seed can not be decrypted!");

  return seed;
}

async function getWallets(chainType: ChainType): Promise<Wallets> {
  if (chainType === "local") {
    const testWallets: Wallets = JSON.parse(
      await readFile(PATH.TO_TEST_WALLETS_PUBLIC, { encoding: ENCODING })
    );

    return testWallets;
  }

  const keyPath = rootPath(PATH.TO_ENCRYPTION_KEY);
  let testWallets: Wallets = JSON.parse(
    await readFile(PATH.TO_TEST_WALLETS, { encoding: ENCODING })
  );

  await access(keyPath);
  const encryptionKey = await readFile(keyPath, { encoding: ENCODING });

  for (const [k, v] of Object.entries(testWallets)) {
    const seed = decrypt(v, encryptionKey);
    if (!seed) throw new Error("Can not get seed!");

    testWallets = { ...testWallets, ...{ [k]: seed } };
  }

  return testWallets;
}

function getSnapshotPath(name: string, chainType: ChainType, fileName: string) {
  return rootPath(
    `./src/backend/services/snapshots/${name}/${chainType}net/${fileName}`
  );
}

/**
 * Converts a Unix epoch time (in seconds) to a human-readable date string in the format "DD.MM.YYYY HH:MM:SS".
 * @param unixTimestamp Unix epoch time in seconds
 * @returns Human-readable date string in the format "DD.MM.YYYY HH:MM:SS"
 */
function epochToDateString(unixTimestamp: number): string {
  const date = new Date(unixTimestamp * 1000);
  const day = date.getDate().toString().padStart(2, "0");
  const month = (date.getMonth() + 1).toString().padStart(2, "0");
  const year = date.getFullYear();
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  const seconds = date.getSeconds().toString().padStart(2, "0");

  return `${day}.${month}.${year} ${hours}:${minutes}:${seconds}`;
}

/**
 * Converts a human-readable date string in the format "DD.MM.YYYY HH:MM:SS" to a Unix epoch time (in seconds).
 * @param dateString Human-readable date string in the format "DD.MM.YYYY HH:MM:SS"
 * @returns Unix epoch time in seconds
 */
function dateStringToEpoch(dateString: string): number {
  const [date, time] = dateString.split(" ");
  const [day, month, year] = date.split(".");
  const [hours, minutes, seconds] = time.split(":");
  const timestamp = new Date(
    parseInt(year),
    parseInt(month) - 1,
    parseInt(day),
    parseInt(hours),
    parseInt(minutes),
    parseInt(seconds)
  );

  return Math.floor(timestamp.getTime() / 1000);
}

/**
 * Converts a Unix epoch time (in seconds) to a human-readable date string in the format "DD.MM.YYYY HH:MM:SS" (UTC).
 * @param unixTimestamp Unix epoch time in seconds
 * @returns Human-readable date string in the format "DD.MM.YYYY HH:MM:SS" (UTC)
 */
function epochToDateStringUTC(unixTimestamp: number): string {
  const date = new Date(unixTimestamp * 1000);
  const day = date.getUTCDate().toString().padStart(2, "0");
  const month = (date.getUTCMonth() + 1).toString().padStart(2, "0");
  const year = date.getUTCFullYear();
  const hours = date.getUTCHours().toString().padStart(2, "0");
  const minutes = date.getUTCMinutes().toString().padStart(2, "0");
  const seconds = date.getUTCSeconds().toString().padStart(2, "0");

  return `${day}.${month}.${year} ${hours}:${minutes}:${seconds}`;
}

/**
 * Converts a human-readable date string in the format "DD.MM.YYYY HH:MM:SS" to a Unix epoch time (in seconds) (UTC).
 * @param dateString Human-readable date string in the format "DD.MM.YYYY HH:MM:SS"
 * @returns Unix epoch time in seconds (UTC)
 */
function dateStringToEpochUTC(dateString: string): number {
  const [date, time] = dateString.split(" ");
  const [day, month, year] = date.split(".");
  const [hours, minutes, seconds] = time.split(":");
  const timestamp = new Date(
    Date.UTC(
      parseInt(year),
      parseInt(month) - 1,
      parseInt(day),
      parseInt(hours),
      parseInt(minutes),
      parseInt(seconds)
    )
  );

  return Math.floor(timestamp.getTime() / 1000);
}

async function specifyTimeout(
  promise: Promise<any>,
  timeout: number = 5_000,
  exception: Function = () => {
    throw new Error("Timeout!");
  }
) {
  let timer: NodeJS.Timeout;

  return Promise.race([
    promise,
    new Promise((_r, rej) => (timer = setTimeout(rej, timeout, exception))),
  ]).finally(() => clearTimeout(timer));
}

function getLocalBlockTime(): number {
  return floor(Date.now() / 1e3);
}

function getBlockTime(blockTimeOffset: number): number {
  return blockTimeOffset + getLocalBlockTime();
}

async function wrapListQuerier<T>(
  fn: (...args: any) => Promise<T[]>,
  ...args: any
): Promise<T[]> {
  try {
    return await fn(...args);
  } catch (error) {
    l(error);
    return [];
  }
}

function getStargazeCollections(
  collectionList: string[],
  mainnetCollectionEntries: [string, NetworkItem][],
  collectionEntries: [string, NetworkItem][]
) {
  const collectionNameList = collectionEntries.filter(
    ([_name, networkItem]) => {
      const address = networkItem?.NEUTRON || "";
      return collectionList.includes(address);
    }
  );

  const stargazeCollectionList = collectionNameList
    .map(([name, _networkItem]) => {
      return (
        mainnetCollectionEntries.find(
          ([currentName]) => currentName === name
        )?.[1]?.STARGAZE || ""
      );
    })
    .filter((address) => address);

  if (collectionList.length != stargazeCollectionList.length) {
    throw new Error("Wrong collection map!");
  }

  return stargazeCollectionList;
}

export {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  decryptSeed,
  parseChainId,
  parseStoreArgs,
  getWallets,
  getSnapshotPath,
  epochToDateString,
  dateStringToEpoch,
  epochToDateStringUTC,
  dateStringToEpochUTC,
  specifyTimeout,
  getLocalBlockTime,
  getBlockTime,
  wrapListQuerier,
  getStargazeCollections,
};
