import { ChainType, Wallets, StoreArgs } from "../../common/interfaces";
declare const ENCODING = "utf8";
declare const PATH_TO_CONFIG_JSON: string;
declare function parseStoreArgs(): StoreArgs;
declare function parseChainId(): string;
declare function decryptSeed(seedEncrypted: string): Promise<string>;
declare function getWallets(chainType: ChainType): Promise<Wallets>;
export { ENCODING, PATH_TO_CONFIG_JSON, decryptSeed, parseChainId, parseStoreArgs, getWallets, };
