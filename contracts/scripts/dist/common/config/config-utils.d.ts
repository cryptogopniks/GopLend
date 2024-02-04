import { ChainConfig, ChainType, Wasm, ChainOption, ContractInfo, ChainItem } from "../interfaces";
declare const $: (value: string) => any;
declare function toJson<T>(obj: T): string;
type MsgType = "instantiate" | "update" | "migrate";
declare function replaceTemplates(configJsonObj: ChainConfig, config: ChainConfig, msgType: MsgType): ChainConfig;
declare function getChain(chainConfig: ChainConfig, name: string): ChainItem;
declare function getChainOption(chainConfig: ChainConfig, name: string, type: ChainType): ChainOption;
declare function getChainOptionById(chainConfig: ChainConfig, chainId: string): {
    NAME: string;
    PREFIX: string;
    OPTION: ChainOption;
};
declare function getContract(chainConfig: ChainConfig, name: string, type: ChainType, wasm: Wasm): ContractInfo;
declare function getContractByWasm(contracts: ContractInfo[], wasm: Wasm): ContractInfo;
export { $, toJson, replaceTemplates, getChain, getChainOption, getContract, getChainOptionById, getContractByWasm, };
