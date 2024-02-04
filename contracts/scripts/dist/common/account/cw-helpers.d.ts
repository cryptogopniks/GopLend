import { DirectSecp256k1HdWallet, OfflineSigner, OfflineDirectSigner } from "@cosmjs/proto-signing";
import { UpdateConfigStruct } from "../interfaces";
import { RawPriceItem } from "../codegen/Oracle.types";
declare function getCwExecHelpers(chainId: string, rpc: string, owner: string, signer: (OfflineSigner & OfflineDirectSigner) | DirectSecp256k1HdWallet): Promise<{
    cwUpdatePrices: (data: RawPriceItem[], gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
    cwUpdateConfig: (updateConfigStruct: UpdateConfigStruct, gasPrice: string) => Promise<import("@cosmjs/cosmwasm-stargate").DeliverTxResponse>;
}>;
declare function getCwQueryHelpers(chainId: string, rpc: string): Promise<{
    cwQueryLendingPlatformConfig: () => Promise<import("../codegen/LendingPlatform.types").Config>;
    cwQueryPrices: (collectionAddresses?: string[]) => Promise<import("../codegen/Oracle.types").ArrayOfQueryPricesResponseItem>;
    cwQueryOracleConfig: () => Promise<import("../codegen/Oracle.types").Config>;
}>;
export { getCwExecHelpers, getCwQueryHelpers };
