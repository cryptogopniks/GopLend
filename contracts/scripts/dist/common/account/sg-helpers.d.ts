import { VoteOption } from "cosmjs-types/cosmos/gov/v1beta1/gov";
import { DirectSecp256k1HdWallet, OfflineSigner, OfflineDirectSigner } from "@cosmjs/proto-signing";
import { Coin } from "@cosmjs/stargate";
declare function getSgExecHelpers(rpc: string, owner: string, signer: (OfflineSigner & OfflineDirectSigner) | DirectSecp256k1HdWallet): Promise<{
    sgDelegate: (operatorAddress: string, amount: number, denom: string, gasPrice: string) => Promise<import("@cosmjs/stargate").DeliverTxResponse>;
    sgVote: (proposalId: number, voteOption: VoteOption, gasPrice: string) => Promise<import("@cosmjs/stargate").DeliverTxResponse>;
    sgSend: (recipient: string, amount: Coin, gasPrice: string) => Promise<import("@cosmjs/stargate").DeliverTxResponse>;
}>;
declare function getSgQueryHelpers(rpc: string): Promise<{
    getAllBalances: (address: string) => Promise<import("cosmjs-types/cosmos/base/v1beta1/coin").Coin[]>;
    getMetadata: (denom: string) => Promise<import("cosmjs-types/cosmos/bank/v1beta1/bank").Metadata>;
    getValidators: () => Promise<import("cosmjs-types/cosmos/staking/v1beta1/staking").Validator[]>;
}>;
export { getSgExecHelpers, getSgQueryHelpers };
