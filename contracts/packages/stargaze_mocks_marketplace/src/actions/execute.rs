use cosmwasm_std::{to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg};

use stargaze_marketplace_v2::{
    orders::OrderDetails,
    state::{OrderId, TokenId},
};

use goplend_base::{
    error::ContractError,
    utils::{check_funds, get_transfer_msg, FundsType},
};

use crate::state::{Bid, COLLECTION_OFFERS, ROYALTY_REGISTRY, TOKEN};

#[allow(clippy::too_many_arguments)]
pub fn accept_collection_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: OrderId,
    token_id: TokenId,
    _details: OrderDetails<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let mut msg_list: Vec<CosmosMsg> = vec![];
    let mut bids = COLLECTION_OFFERS.load(deps.storage)?;

    let current_bid = bids
        .iter()
        .find(|x| x.id == id)
        .ok_or(ContractError::CustomError {
            val: "bid isn't found".to_string(),
        })?;

    msg_list.push(get_transfer_msg(
        &sender_address,
        current_bid.price,
        &TOKEN.load(deps.storage)?,
    )?);

    // transfer nft to liquidator
    msg_list.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: current_bid.collection.to_string(),
        msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
            recipient: current_bid.bidder.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    }));

    bids.retain(|x| x.id != id);
    COLLECTION_OFFERS.save(deps.storage, &bids)?;

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "accept_collection_bid")]))
}

pub fn try_set_bids(deps: DepsMut, _env: Env, bids: Vec<Bid>) -> Result<Response, ContractError> {
    COLLECTION_OFFERS.save(deps.storage, &bids)?;

    Ok(Response::new().add_attributes(vec![("action", "try_set_bids")]))
}

pub fn try_set_royalty_registry(
    deps: DepsMut,
    _env: Env,
    royalty_registry: String,
) -> Result<Response, ContractError> {
    let royalty_registry = deps.api.addr_validate(&royalty_registry)?;
    ROYALTY_REGISTRY.save(deps.storage, &royalty_registry)?;

    Ok(Response::new().add_attributes(vec![("action", "try_set_bids")]))
}
