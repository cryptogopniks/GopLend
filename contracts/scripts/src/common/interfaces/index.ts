import { Coin } from "@cosmjs/stargate";
import { Timestamp } from "cosmjs-types/google/protobuf/timestamp";
import { NetworkName, Wasm } from "../config";

import {
  CurrencyForTokenUnverified,
  Decimal,
} from "../codegen/LendingPlatform.types";

interface QueryContractResponse {
  address: string;
  contract_info: {
    code_id: string;
    creator: string;
    admin: string;
    label: string;
    created: {
      block_height: string;
      tx_index: string;
    };
    ibc_port_id: string;
    extension: {
      type_url: string;
      value: string;
    };
  };
}

interface QueryCodeIdsResponse {
  code_infos: {
    code_id: string;
    creator: string;
    data_hash: string;
    instantiate_permission: {
      permission: string;
      address: string;
      addresses: string[];
    };
  }[];
  pagination: {
    next_key: string;
    total: string;
  };
}

// pagination field is ignored
interface QueryProposalsResponse {
  proposals: {
    proposal_id: string;
    content: {
      "@type": string;
      title: string;
      description: string;
      run_as: string;
      wasm_byte_code: string;
      instantiate_permission: {
        permission: string;
        address: string;
        addresses: string[];
      };
      unpin_code: boolean;
      source: string;
      builder: string;
      code_hash: string;
    };
    status: string;
    final_tally_result: {
      yes: string;
      abstain: string;
      no: string;
      no_with_veto: string;
    };
    submit_time: Timestamp;
    deposit_end_time: Timestamp;
    total_deposit: Coin[];
    voting_start_time: Timestamp;
    voting_end_time: Timestamp;
  }[];
}

interface Wallets {
  SEED_ADMIN: string;
  SEED_ALICE: string;
  SEED_BOB: string;
}

interface StoreArgs {
  chainId: string;
  wasmList: Wasm[];
}

interface MintStruct {
  amount: number;
  recipient: string;
  channelId?: string;
}

interface MintOverIbcStruct {
  contractAddress: string;
  amount: number;
  recipient: string;
  channelId?: string;
}

interface Cw20SendMsg {
  send: {
    contract: string;
    amount: string;
    msg: string;
  };
}

interface SetMetadataMsg {
  set_metadata: {
    metadata: Metadata;
  };
}

interface LocalInterchainLogs {
  start_time: number;
  chains: {
    chain_id: string;
    chain_name: string;
    rpc_address: string;
    rest_address: string;
    grpc_address: string;
    ibc_paths: string[];
  }[];
  ibc_channels: {
    chain_id: string;
    channel: {
      state: string;
      ordering: string;
      counterparty: {
        port_id: string;
        channel_id: string;
      };
      connection_hops: string[];
      version: string;
      port_id: string;
      channel_id: string;
    };
  }[];
}

type ChainType = "main" | "test" | "local";

interface ContractInfo {
  WASM: Wasm;
  LABEL: string;
  INIT_MSG: string;
  MIGRATE_MSG: string;
  UPDATE_MSG: string;
  CODE: number;
  ADDRESS: string;
}

interface ChainOption {
  TYPE: ChainType;
  DENOM: string;
  CHAIN_ID: string;
  RPC_LIST: string[];
  GAS_PRICE_AMOUNT: number;
  STORE_CODE_GAS_MULTIPLIER: number;
  CONTRACTS: ContractInfo[];
  IBC: IbcConfig[];
}

interface IbcConfig {
  COUNTERPARTY_CHAIN_ID: string;
  CHANNEL_ID: string;
  PORT: string;
}

interface ChainItem {
  NAME: string;
  PREFIX: string;
  OPTIONS: ChainOption[];
}

interface ChainConfig {
  CHAINS: ChainItem[];
}

type TokenUnverified =
  | {
      native: {
        denom: string;
      };
    }
  | {
      cw20: {
        address: string;
      };
    };

interface Pagination {
  next_key: string;
  total: string;
}

interface QueryContractCodesResponse {
  code_infos: {
    code_id: string;
    creator: string;
    data_hash: string;
    instantiate_permission: {
      permission: string;
      address: string;
      addresses: string[];
    };
  }[];
  pagination: Pagination;
}

interface Cw20SendMsg {
  send: {
    contract: string;
    amount: string;
    msg: string;
  };
}

interface Metadata {
  base: string;
  denom_units: { aliases: string[]; denom: string; exponent: string }[];
  description: string;
  display: string;
  name: string;
  symbol: string;
  uri?: string;
  uri_hash?: string;
}

interface SetMetadataMsg {
  set_metadata: {
    metadata: Metadata;
  };
}

interface QueryAllOperatorsMsg {
  all_operators: {
    owner: string;
    include_expired?: boolean;
    start_after?: string;
    limit?: number;
  };
}

interface QueryAllOperatorsInjMsg {
  approved_for_all: {
    owner: string;
    include_expired?: boolean;
    start_after?: string;
    limit?: number;
  };
}

interface QueryAllOperatorsResponse {
  operators: Approval[];
}

interface QueryAllOperatorsInjResponse {
  approvals: Approval[];
}

interface ApproveAllMsg {
  approve_all: {
    operator: string;
    expires?: Expiration;
  };
}

interface ApproveMsg {
  approve: {
    spender: string;
    token_id: string;
    expires?: Expiration;
  };
}

interface RevokeAllMsg {
  revoke_all: { operator: string };
}

interface RevokeMsg {
  revoke: {
    spender: string;
    token_id: string;
  };
}

interface QueryApprovalsMsg {
  approvals: {
    token_id: string;
    include_expired?: boolean;
  };
}

interface Approval {
  spender: string;
  expires: Expiration;
}

type Expiration =
  | { at_height: number }
  | { at_time: Timestamp }
  | { never: {} };

interface ApprovalsResponse {
  approvals: Approval[];
}

interface QueryTokens {
  tokens: {
    owner: string;
    start_after?: string;
    limit?: number;
  };
}

interface TokensResponse {
  tokens: string[];
}

interface TokensResponseInj {
  ids: string[];
}

interface QueryOwnerOf {
  owner_of: {
    token_id: string;
    include_expired?: boolean;
  };
}

interface OwnerOfResponse {
  owner: string;
  approvals: Approval[];
}

type NetworkConfig = {
  [network in NetworkName]: {
    BASE: BaseNetworkConfig;
    CONTRACTS: ContractsConfig[];
  };
};

type BaseNetworkConfig = {
  PREFIX: string;
  DENOM: string;
  CHAIN_ID: string;
  RPC_LIST: string[];
  GAS_PRICE_AMOUNT: number;
  STORE_CODE_GAS_MULTIPLIER: number;
};

type ContractsConfig = {
  WASM: string;
  LABEL: string;
  INIT_MSG: any;
  MIGRATE_MSG: any;
  DATA: ContractData;
};

type ContractData = {
  CODE: number;
  ADDRESS: string;
};

export type {
  TokenUnverified,
  ChainType,
  Wasm,
  NetworkConfig,
  NetworkName,
  ContractsConfig,
};

export {
  SetMetadataMsg,
  Metadata,
  Cw20SendMsg,
  MintStruct,
  MintOverIbcStruct,
  ChainConfig,
  IbcConfig,
  ChainOption,
  LocalInterchainLogs,
  ContractInfo,
  ChainItem,
  Wallets,
  StoreArgs,
  QueryProposalsResponse,
  QueryCodeIdsResponse,
  QueryContractResponse,
  QueryApprovalsMsg,
  ApprovalsResponse,
  BaseNetworkConfig,
  ContractData,
  QueryAllOperatorsMsg,
  QueryAllOperatorsResponse,
  QueryAllOperatorsInjMsg,
  QueryAllOperatorsInjResponse,
  ApproveAllMsg,
  ApproveMsg,
  RevokeAllMsg,
  RevokeMsg,
  QueryTokens,
  TokensResponse,
  TokensResponseInj,
  QueryOwnerOf,
  OwnerOfResponse,
  QueryContractCodesResponse,
};
