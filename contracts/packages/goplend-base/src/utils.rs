use std::fmt::Debug;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, to_json_binary, Addr, Api, BankMsg, Coin, CosmosMsg, Deps, MessageInfo, QuerierWrapper,
    StdError, StdResult, Uint128, WasmMsg,
};

use crate::{assets::Token, error::ContractError};

#[cw_serde]
pub enum FundsType {
    Empty,
    Single {
        sender: Option<String>,
        amount: Option<Uint128>,
    },
}

#[cw_serde]
pub enum AuthType {
    Any,
    Admin,
    AdminOrWorker,
    Specified { allowlist: Vec<Option<Addr>> },
    AdminOrWorkerOrSpecified { allowlist: Vec<Option<Addr>> },
    AdminOrSpecified { allowlist: Vec<Option<Addr>> },
    Excluded { ignorelist: Vec<Option<Addr>> },
}

pub fn check_authorization(
    sender: &Addr,
    admin: &Addr,
    worker: &Option<Addr>,
    auth_type: AuthType,
) -> StdResult<()> {
    let worker = unwrap_field(worker.to_owned(), "worker");

    match auth_type {
        AuthType::Any => {}
        AuthType::Admin => {
            if sender != admin {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorker => {
            if !((sender == admin) || (worker.is_ok() && sender == worker?)) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::Specified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !is_included {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrWorkerOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || (worker.is_ok() && sender == worker?) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::AdminOrSpecified { allowlist } => {
            let is_included = allowlist.iter().any(|some_address| {
                if let Some(x) = some_address {
                    if sender == x {
                        return true;
                    }
                }

                false
            });

            if !((sender == admin) || is_included) {
                Err(ContractError::Unauthorized)?;
            }
        }
        AuthType::Excluded { ignorelist } => {
            if ignorelist.contains(&Some(sender.to_owned())) {
                Err(ContractError::Unauthorized)?;
            }
        }
    };

    Ok(())
}

#[cw_serde]
pub struct Attrs {}

impl Attrs {
    pub fn init(action: &str) -> Vec<(String, String)> {
        vec![("action".to_string(), action.to_string())]
    }
}

pub fn add_attr<T: Debug + Clone>(
    attrs: &mut Vec<(String, String)>,
    attr: &str,
    field: &Option<T>,
) -> StdResult<Option<T>> {
    if let Some(x) = field {
        attrs.push((attr.to_string(), format!("{:#?}", x)));

        return Ok(Some(x.to_owned()));
    }

    Ok(None)
}

pub fn validate_attr(
    attrs: &mut Vec<(String, String)>,
    api: &dyn Api,
    attr: &str,
    field: &Option<String>,
) -> StdResult<Option<Addr>> {
    if let Some(x) = field {
        let value = api.addr_validate(x)?;
        attrs.push((attr.to_string(), value.to_string()));

        return Ok(Some(value));
    }

    Ok(None)
}

pub fn unwrap_field<T>(field: Option<T>, name: &str) -> Result<T, ContractError> {
    field.ok_or(ContractError::ParameterIsNotFound {
        value: name.to_string(),
    })
}

pub fn add_funds_to_exec_msg(
    exec_msg: &WasmMsg,
    funds_list: &[(Uint128, Token)],
) -> StdResult<WasmMsg> {
    let mut native_tokens: Vec<Coin> = vec![];
    let mut cw20_tokens: Vec<(Uint128, Addr)> = vec![];

    for (amount, token) in funds_list.iter().cloned() {
        match token {
            Token::Native { denom } => {
                native_tokens.push(coin(amount.u128(), denom));
            }
            Token::Cw20 { address } => {
                cw20_tokens.push((amount, address));
            }
        }
    }

    match exec_msg {
        WasmMsg::Execute {
            contract_addr, msg, ..
        } => {
            // Case 1 `Deposit` - only native tokens
            if cw20_tokens.is_empty() {
                return Ok(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: msg.to_owned(),
                    funds: native_tokens,
                });
            }

            // Case 2 `Swap` - only single cw20 token
            if (cw20_tokens.len() == 1) && native_tokens.is_empty() {
                let (amount, token_address) =
                    cw20_tokens.get(0).ok_or(ContractError::AssetIsNotFound)?;

                return Ok(WasmMsg::Execute {
                    contract_addr: token_address.to_string(),
                    msg: to_json_binary(&cw20::Cw20ExecuteMsg::Send {
                        contract: contract_addr.to_string(),
                        amount: amount.to_owned(),
                        msg: msg.to_owned(),
                    })?,
                    funds: vec![],
                });
            }

            Err(ContractError::WrongFundsCombination)?
        }
        _ => Err(ContractError::WrongActionType)?,
    }
}

pub fn get_transfer_msg(recipient: &Addr, amount: Uint128, token: &Token) -> StdResult<CosmosMsg> {
    Ok(match token {
        Token::Native { denom } => CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![coin(amount.u128(), denom)],
        }),
        Token::Cw20 { address } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address.to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount: amount.to_owned(),
            })?,
            funds: vec![],
        }),
    })
}

/// Returns (sender_address, asset_amount, asset_info) supporting both native and cw20 tokens \
/// Use FundsType::Empty to check if info.funds is empty \
/// Use FundsType::Single { sender: None, amount: None } to check native token \
/// Use FundsType::Single { sender: Some(msg.sender), amount: Some(msg.amount) } to check cw20 token
pub fn check_funds(
    deps: Deps,
    info: &MessageInfo,
    funds_type: FundsType,
) -> StdResult<(Addr, Uint128, Token)> {
    match funds_type {
        FundsType::Empty => {
            nonpayable(info)?;

            Ok((
                info.sender.clone(),
                Uint128::default(),
                Token::new_native(&String::default()),
            ))
        }
        FundsType::Single { sender, amount } => {
            if sender.is_none() || amount.is_none() {
                let Coin { denom, amount } = one_coin(info)?;

                Ok((info.sender.clone(), amount, Token::new_native(&denom)))
            } else {
                Ok((
                    deps.api
                        .addr_validate(&sender.ok_or(ContractError::WrongFundsCombination)?)?,
                    amount.ok_or(ContractError::WrongFundsCombination)?,
                    Token::new_cw20(&info.sender),
                ))
            }
        }
    }
}

/// If exactly one coin was sent, returns it regardless of denom.
/// Returns error if 0 or 2+ coins were sent
fn one_coin(info: &MessageInfo) -> StdResult<Coin> {
    if info.funds.len() == 1 {
        let coin = &info.funds[0];

        if !coin.amount.is_zero() {
            return Ok(coin.to_owned());
        }

        Err(StdError::generic_err("Coins amount is zero!"))?;
    }

    Err(StdError::generic_err("Amount of denoms is not equal 1!"))?
}

/// returns an error if any coins were sent
fn nonpayable(info: &MessageInfo) -> StdResult<()> {
    if !info.funds.is_empty() {
        Err(StdError::generic_err("This message does no accept funds!"))?;
    }

    Ok(())
}

pub fn get_collection_operator_approvals(
    querier: QuerierWrapper,
    collection_list: &[impl ToString],
    owner: impl ToString,
    operator: impl ToString,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msg_list: Vec<CosmosMsg> = vec![];

    for collection in collection_list {
        let cw721::OperatorsResponse { operators } = querier.query_wasm_smart(
            collection.to_string(),
            &cw721::Cw721QueryMsg::AllOperators {
                owner: owner.to_string(),
                include_expired: None,
                start_after: None,
                limit: None,
            },
        )?;

        let target_operator = operators.iter().find(|x| x.spender == operator.to_string());

        if target_operator.is_none() {
            msg_list.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection.to_string(),
                msg: to_json_binary(&cw721::Cw721ExecuteMsg::ApproveAll {
                    operator: operator.to_string(),
                    expires: None,
                })?,
                funds: vec![],
            }));
        }
    }

    Ok(msg_list)
}
