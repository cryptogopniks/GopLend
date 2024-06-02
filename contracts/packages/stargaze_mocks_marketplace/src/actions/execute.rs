use cosmwasm_std::{to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg};

use goplend_base::{
    error::ContractError,
    utils::{check_funds, get_transfer_msg, FundsType},
};
use sg_marketplace::state::TokenId;

use crate::state::{COLLECTION_OFFERS, TOKEN};

#[allow(clippy::too_many_arguments)]
pub fn accept_collection_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    token_id: TokenId,
    bidder: String,
    _finder: Option<String>,
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

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection.clone(),
        msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
            recipient: env.contract.address.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    });

    msg_list.push(msg);

    bids.retain(|x| !(x.bidder == bidder && x.collection == collection));
    COLLECTION_OFFERS.save(deps.storage, &bids)?;

    Ok(Response::new()
        .add_messages(msg_list)
        .add_attributes(vec![("action", "accept_collection_bid")]))
}
