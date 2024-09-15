use cosmwasm_std::{
    coins, to_json_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage, Uint128, WasmMsg,
};

use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;

use goplend_base::{
    assets::TokenUnverified,
    error::ContractError,
    lending_platform::types::CollectionInfo,
    market_maker::{
        outpost::state::CONFIG,
        types::{LiquidityInfo, OffersResponse, OutpostConfig, TransferAdminState},
    },
    utils::{
        check_authorization, check_funds, get_collection_operator_approvals, get_transfer_msg,
        unwrap_field, AuthType, FundsType,
    },
};
