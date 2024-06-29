import { access, readFile } from "fs/promises";
import { decrypt, getLast } from "../../common/utils";
import { PATH, rootPath } from "../envs";
import { ChainType, Wallets, StoreArgs } from "../../common/interfaces";
import { Label } from "../../common/config";

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

export {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  decryptSeed,
  parseChainId,
  parseStoreArgs,
  getWallets,
};
