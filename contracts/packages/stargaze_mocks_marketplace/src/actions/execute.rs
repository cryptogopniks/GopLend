use cosmwasm_std::{
    to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};

use goplend_base::{
    error::ContractError,
    utils::{check_funds, get_transfer_msg, FundsType},
};
use sg_marketplace::state::TokenId;

use crate::state::{Bid, COLLECTION_OFFERS, TOKEN};

#[allow(clippy::too_many_arguments)]
pub fn accept_collection_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    token_id: TokenId,
    bidder: String,
    _finder: Option<String>,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;

    let mut msg_list: Vec<CosmosMsg> = vec![];
    let mut bids = COLLECTION_OFFERS.load(deps.storage)?;

    let current_bid = bids
        .iter()
        .find(|x| x.bidder == bidder && x.collection == collection)
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
        contract_addr: collection.clone(),
        msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
            recipient: current_bid.bidder.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    }));

    bids.retain(|x| !(x.bidder == bidder && x.collection == collection));
    COLLECTION_OFFERS.save(deps.storage, &bids)?;

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "accept_collection_bid")]))
}

pub fn try_set_bids(deps: DepsMut, _env: Env, bids: Vec<Bid>) -> Result<Response, ContractError> {
    COLLECTION_OFFERS.save(deps.storage, &bids)?;

    Ok(Response::new().add_attributes(vec![("action", "try_set_bids")]))
}
