use anybuf::Anybuf;
use bech32::{decode, encode, Bech32, Hrp};

use cosmwasm_std::{
    coin, to_json_string, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};

use skip::{
    asset::Asset,
    entry_point::Action,
    swap::{Swap, SwapExactAssetIn, SwapOperation},
};

use goplend_base::{
    adapters::dex::stargaze::{
        state::{
            CHANNEL_NEUTRON_NOBLE, CHANNEL_NEUTRON_OSMOSIS, CHANNEL_NOBLE_NEUTRON,
            CHANNEL_NOBLE_STARGAZE, CHANNEL_OSMOSIS_NOBLE, CHANNEL_STARGAZE_NEUTRON,
            CHANNEL_STARGAZE_OSMOSIS, CONFIG, CONTRACT_NEUTRON_SKIP_ENTRY_POINT,
            CONTRACT_OSMOSIS_SKIP_ENTRY_POINT, DENOM_NEUTRON_NTRN, DENOM_NEUTRON_STARS,
            DENOM_NEUTRON_USDC, DENOM_OSMOSIS_NTRN, DENOM_OSMOSIS_OSMO, DENOM_OSMOSIS_STARS,
            DENOM_OSMOSIS_USDC, IBC_TIMEOUT, MAX_PRICE_IMPACT, POOL_NEUTRON_NTRN_USDC,
            POOL_NEUTRON_STARS_NTRN, POOL_OSMOSIS_NTRN_OSMO, POOL_OSMOSIS_OSMO_USDC,
            POOL_OSMOSIS_STARS_OSMO, TRANSFER_ADMIN_STATE, TRANSFER_ADMIN_TIMEOUT,
        },
        types::{Config, Route, TransferAdminState},
    },
    converters::{str_to_dec, u128_to_dec},
    error::ContractError,
    utils::{check_funds, unwrap_field, AuthType, FundsType},
};

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let block_time = env.block.time.seconds();
    let TransferAdminState {
        new_admin,
        deadline,
    } = TRANSFER_ADMIN_STATE.load(deps.storage)?;

    check_authorization(
        deps.as_ref(),
        &sender_address,
        AuthType::Specified {
            allowlist: vec![Some(new_admin)],
        },
    )?;

    if block_time >= deadline {
        Err(ContractError::TransferAdminDeadline)?;
    }

    CONFIG.update(deps.storage, |mut x| -> StdResult<Config> {
        x.admin = sender_address;
        Ok(x)
    })?;

    TRANSFER_ADMIN_STATE.update(deps.storage, |mut x| -> StdResult<TransferAdminState> {
        x.deadline = block_time;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_accept_admin_role"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    lending_platform_stable: Option<String>,
    adapter_marketplace: Option<String>,
    recover_address: Option<String>,
    max_price_impact: Option<Decimal>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;

    if let Some(x) = admin {
        check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;
        let block_time = env.block.time.seconds();
        let new_admin = &deps.api.addr_validate(&x)?;

        TRANSFER_ADMIN_STATE.save(
            deps.storage,
            &TransferAdminState {
                new_admin: new_admin.to_owned(),
                deadline: block_time + TRANSFER_ADMIN_TIMEOUT,
            },
        )?;

        is_config_updated = true;
    }

    if let Some(x) = worker {
        check_authorization(deps.as_ref(), &sender_address, AuthType::Admin)?;
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = lending_platform_stable {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.lending_platform_stable = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = adapter_marketplace {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.adapter_marketplace = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = recover_address {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.recover_address = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = max_price_impact {
        check_authorization(deps.as_ref(), &sender_address, AuthType::AdminOrWorker)?;
        config.max_price_impact = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    route: Route,
    price_ratio: Decimal,
) -> Result<Response, ContractError> {
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    let Config {
        lending_platform_stable,
        adapter_marketplace,
        recover_address,
        ..
    } = CONFIG.load(deps.storage)?;
    // check_authorization(
    //     deps.as_ref(),
    //     &sender_address,
    //     AuthType::Specified {
    //         allowlist: vec![adapter_marketplace],
    //     },
    // )?;

    let lending_platform_stable =
        &unwrap_field(lending_platform_stable, "lending_platform_stable")?;
    let recover_address = &unwrap_field(recover_address, "recover_address")?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    let denom_in = &asset_info.try_get_native()?;
    let max_price_impact = str_to_dec(MAX_PRICE_IMPACT);
    let timeout_timestamp_ns = env.block.time.plus_seconds(IBC_TIMEOUT).nanos();

    let msg = match route {
        Route::Osmosis => {
            let ibc_swap_memo = get_osmosis_ibc_swap_memo(
                asset_amount,
                price_ratio,
                max_price_impact,
                timeout_timestamp_ns,
                lending_platform_stable,
                recover_address,
            );

            get_ibc_swap_msg(
                CHANNEL_STARGAZE_OSMOSIS,
                denom_in,
                asset_amount,
                &sender_address,
                CONTRACT_OSMOSIS_SKIP_ENTRY_POINT,
                timeout_timestamp_ns,
                &ibc_swap_memo,
            )
        }
        Route::Neutron => {
            let ibc_swap_memo = get_neutron_ibc_swap_memo(
                asset_amount,
                price_ratio,
                max_price_impact,
                timeout_timestamp_ns,
                lending_platform_stable,
                recover_address,
            );

            get_ibc_swap_msg(
                CHANNEL_STARGAZE_NEUTRON,
                denom_in,
                asset_amount,
                &sender_address,
                CONTRACT_NEUTRON_SKIP_ENTRY_POINT,
                timeout_timestamp_ns,
                &ibc_swap_memo,
            )
        }
        Route::Test => {
            let ibc_swap_memo = get_swap_and_action_memo(
                asset_amount,
                price_ratio,
                max_price_impact,
                timeout_timestamp_ns,
                lending_platform_stable,
                recover_address,
            )?;

            get_ibc_swap_msg(
                CHANNEL_NEUTRON_OSMOSIS,
                denom_in,
                asset_amount,
                &sender_address,
                CONTRACT_OSMOSIS_SKIP_ENTRY_POINT,
                timeout_timestamp_ns,
                &ibc_swap_memo,
            )
        }
        // Route::Test2 => {
        //     let ibc_swap_memo = get_test_2_ibc_swap_memo(
        //         asset_amount,
        //         price_ratio,
        //         max_price_impact,
        //         timeout_timestamp_ns,
        //         lending_platform_stable,
        //         recover_address,
        //     );

        //     get_ibc_swap_msg(
        //         CHANNEL_NEUTRON_OSMOSIS,
        //         denom_in,
        //         asset_amount,
        //         &sender_address,
        //         CONTRACT_OSMOSIS_SKIP_ENTRY_POINT,
        //         timeout_timestamp_ns,
        //         &ibc_swap_memo,
        //     )
        // }
        _ => unimplemented!(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_swap"))
}

fn get_ibc_swap_msg(
    channel: &str,
    denom_in: &str,
    amount_in: Uint128,
    sender: &Addr,
    skip_entry_point: &str,
    timeout_timestamp_ns: u64,
    ibc_swap_memo: &str,
) -> CosmosMsg {
    CosmosMsg::Stargate {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: Anybuf::new()
            // source port
            .append_string(1, "transfer")
            // source channel (IBC Channel on your network side)
            .append_string(2, channel)
            // token
            .append_message(
                3,
                &Anybuf::new()
                    .append_string(1, denom_in)
                    .append_string(2, amount_in.to_string()),
            )
            // sender
            .append_string(4, sender)
            // recipient
            .append_string(5, skip_entry_point)
            // TimeoutHeight
            .append_message(6, &Anybuf::new().append_uint64(1, 0).append_uint64(2, 0))
            // TimeoutTimestamp
            .append_uint64(7, timeout_timestamp_ns)
            // IBC Hook memo
            .append_string(8, ibc_swap_memo)
            .into_vec()
            .into(),
    }
}

fn get_osmosis_ibc_swap_memo(
    token_in_amount: Uint128,
    price_ratio: Decimal,
    max_price_impact: Decimal,
    timeout_timestamp_ns: u64,
    recipient: &Addr,
    recover_address: &Addr,
) -> String {
    // min_token_out_amount = token_in_amount * price_ratio * (1 - max_price_impact)
    let min_token_out_amount =
        (u128_to_dec(token_in_amount) * price_ratio * (Decimal::one() - max_price_impact))
            .to_uint_floor()
            .u128();

    let recipient = &recipient.to_string();
    let recover_address = &recover_address.to_string();
    let recipient_noble = get_addr_by_prefix(recipient, "noble");
    let recover_address_osmosis = get_addr_by_prefix(recover_address, "osmosis");

    format!(
        "{{\"wasm\":{{\"contract\":\"{CONTRACT_OSMOSIS_SKIP_ENTRY_POINT}\",\
        \"msg\":{{\"swap_and_action\":{{\"user_swap\":{{\"swap_exact_asset_in\":\
        {{\"swap_venue_name\":\"osmosis-poolmanager\",\"operations\":[{{\"pool\":\
        \"{POOL_OSMOSIS_STARS_OSMO}\",\"denom_in\":\"{DENOM_OSMOSIS_STARS}\",\
        \"denom_out\":\"{DENOM_OSMOSIS_OSMO}\"}},{{\"pool\":\"{POOL_OSMOSIS_OSMO_USDC}\",\
        \"denom_in\":\"{DENOM_OSMOSIS_OSMO}\",\"denom_out\":\"{DENOM_OSMOSIS_USDC}\"}}]}}}},\
        \"min_asset\":{{\"native\":{{\"denom\":\"{DENOM_OSMOSIS_USDC}\",\
        \"amount\":\"{min_token_out_amount}\"}}}},\"timeout_timestamp\":{timeout_timestamp_ns},\
        \"post_swap_action\":{{\"ibc_transfer\":{{\"ibc_info\":{{\"source_channel\":\
        \"{CHANNEL_OSMOSIS_NOBLE}\",\"receiver\":\"{recipient_noble}\",\"memo\":\"{{\\\"forward\\\":\
        {{\\\"channel\\\":\\\"{CHANNEL_NOBLE_STARGAZE}\\\",\\\"port\\\":\\\"transfer\\\",\
        \\\"receiver\\\":\\\"{recipient}\\\",\\\"retries\\\":2,\\\"timeout\\\":\
        {timeout_timestamp_ns}}}}}\",\"recover_address\":\"{recover_address_osmosis}\"}}}}}},\
        \"affiliates\":[]}}}}}}}}"
    )
}

fn get_neutron_ibc_swap_memo(
    token_in_amount: Uint128,
    price_ratio: Decimal,
    max_price_impact: Decimal,
    timeout_timestamp_ns: u64,
    recipient: &Addr,
    recover_address: &Addr,
) -> String {
    // min_token_out_amount = token_in_amount * price_ratio * (1 - max_price_impact)
    let min_token_out_amount =
        (u128_to_dec(token_in_amount) * price_ratio * (Decimal::one() - max_price_impact))
            .to_uint_floor()
            .u128();

    let recipient = &recipient.to_string();
    let recover_address = &recover_address.to_string();
    let recipient_noble = get_addr_by_prefix(recipient, "noble");
    let recover_address_neutron = get_addr_by_prefix(recover_address, "neutron");

    format!(
        "{{\"wasm\":{{\"contract\":\"{CONTRACT_NEUTRON_SKIP_ENTRY_POINT}\",\
        \"msg\":{{\"swap_and_action\":{{\"user_swap\":{{\"swap_exact_asset_in\":\
        {{\"swap_venue_name\":\"neutron-astroport\",\"operations\":[{{\"pool\":\
        \"{POOL_NEUTRON_STARS_NTRN}\",\"denom_in\":\"{DENOM_NEUTRON_STARS}\",\
        \"denom_out\":\"{DENOM_NEUTRON_NTRN}\"}},{{\"pool\":\"{POOL_NEUTRON_NTRN_USDC}\",\
        \"denom_in\":\"{DENOM_NEUTRON_NTRN}\",\"denom_out\":\"{DENOM_NEUTRON_USDC}\"}}]}}}},\
        \"min_asset\":{{\"native\":{{\"denom\":\"{DENOM_NEUTRON_USDC}\",\
        \"amount\":\"{min_token_out_amount}\"}}}},\"timeout_timestamp\":{timeout_timestamp_ns},\
        \"post_swap_action\":{{\"ibc_transfer\":{{\"ibc_info\":{{\"source_channel\":\
        \"{CHANNEL_NEUTRON_NOBLE}\",\"receiver\":\"{recipient_noble}\",\"memo\":\"{{\\\"forward\\\":\
        {{\\\"channel\\\":\\\"{CHANNEL_NOBLE_STARGAZE}\\\",\\\"port\\\":\\\"transfer\\\",\
        \\\"receiver\\\":\\\"{recipient}\\\",\\\"retries\\\":2,\\\"timeout\\\":\
        {timeout_timestamp_ns}}}}}\",\"recover_address\":\"{recover_address_neutron}\"}}}}}},\
        \"affiliates\":[]}}}}}}}}"
    )
}

fn get_test_ibc_swap_memo(
    token_in_amount: Uint128,
    price_ratio: Decimal,
    max_price_impact: Decimal,
    timeout_timestamp_ns: u64,
    recipient: &Addr,
    recover_address: &Addr,
) -> String {
    // min_token_out_amount = token_in_amount * price_ratio * (1 - max_price_impact)
    let min_token_out_amount =
        (u128_to_dec(token_in_amount) * price_ratio * (Decimal::one() - max_price_impact))
            .to_uint_floor()
            .u128();

    let recipient = &recipient.to_string();
    let recover_address = &recover_address.to_string();
    let recipient_noble = get_addr_by_prefix(recipient, "noble");
    let recover_address_osmosis = get_addr_by_prefix(recover_address, "osmosis");

    format!(
        "{{\"wasm\":{{\"contract\":\"{CONTRACT_OSMOSIS_SKIP_ENTRY_POINT}\",\
        \"msg\":{{\"swap_and_action\":{{\"user_swap\":{{\"swap_exact_asset_in\":\
        {{\"swap_venue_name\":\"osmosis-poolmanager\",\"operations\":[{{\"pool\":\
        \"{POOL_OSMOSIS_NTRN_OSMO}\",\"denom_in\":\"{DENOM_OSMOSIS_NTRN}\",\
        \"denom_out\":\"{DENOM_OSMOSIS_OSMO}\"}},{{\"pool\":\"{POOL_OSMOSIS_OSMO_USDC}\",\
        \"denom_in\":\"{DENOM_OSMOSIS_OSMO}\",\"denom_out\":\"{DENOM_OSMOSIS_USDC}\"}}]}}}},\
        \"min_asset\":{{\"native\":{{\"denom\":\"{DENOM_OSMOSIS_USDC}\",\
        \"amount\":\"{min_token_out_amount}\"}}}},\"timeout_timestamp\":{timeout_timestamp_ns},\
        \"post_swap_action\":{{\"ibc_transfer\":{{\"ibc_info\":{{\"source_channel\":\
        \"{CHANNEL_OSMOSIS_NOBLE}\",\"receiver\":\"{recipient_noble}\",\"memo\":\"{{\\\"forward\\\":\
        {{\\\"channel\\\":\\\"{CHANNEL_NOBLE_NEUTRON}\\\",\\\"port\\\":\\\"transfer\\\",\
        \\\"receiver\\\":\\\"{recipient}\\\",\\\"retries\\\":2,\\\"timeout\\\":\
        {timeout_timestamp_ns}}}}}\",\"recover_address\":\"{recover_address_osmosis}\"}}}}}},\
        \"affiliates\":[]}}}}}}}}"
    )
}

// swap and send on osmosis contract address
fn get_test_2_ibc_swap_memo(
    token_in_amount: Uint128,
    price_ratio: Decimal,
    max_price_impact: Decimal,
    timeout_timestamp_ns: u64,
    _recipient: &Addr,
    _recover_address: &Addr,
) -> String {
    // min_token_out_amount = token_in_amount * price_ratio * (1 - max_price_impact)
    let min_token_out_amount =
        (u128_to_dec(token_in_amount) * price_ratio * (Decimal::one() - max_price_impact))
            .to_uint_floor()
            .u128();

    // let recipient = &recipient.to_string();
    // let recover_address = &recover_address.to_string();
    // let recipient_noble = get_addr_by_prefix(recipient, "noble");
    // let recover_address_osmosis = get_addr_by_prefix(recover_address, "osmosis");

    // https://osmosis-api.lavenderfive.com/cosmos/bank/v1beta1/balances/osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw
    // 5149 ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4
    const RECIPIENT: &str = "osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw";

    format!(
        "{{\"wasm\":{{\"contract\":\"{CONTRACT_OSMOSIS_SKIP_ENTRY_POINT}\",\
        \"msg\":{{\"swap_and_action\":{{\"user_swap\":{{\"swap_exact_asset_in\":\
        {{\"swap_venue_name\":\"osmosis-poolmanager\",\"operations\":[{{\"pool\":\
        \"{POOL_OSMOSIS_NTRN_OSMO}\",\"denom_in\":\"{DENOM_OSMOSIS_NTRN}\",\
        \"denom_out\":\"{DENOM_OSMOSIS_OSMO}\"}},{{\"pool\":\"{POOL_OSMOSIS_OSMO_USDC}\",\
        \"denom_in\":\"{DENOM_OSMOSIS_OSMO}\",\"denom_out\":\"{DENOM_OSMOSIS_USDC}\"}}]}}}},\
        \"min_asset\":{{\"native\":{{\"denom\":\"{DENOM_OSMOSIS_USDC}\",\
        \"amount\":\"{min_token_out_amount}\"}}}},\"timeout_timestamp\":{timeout_timestamp_ns},\
        \"post_swap_action\":{{\"transfer\":{{\"to_address\":\"{RECIPIENT}\"}}}},\
        \"affiliates\":[]}}}}}}}}"
    )
}

fn get_swap_and_action_memo(
    token_in_amount: Uint128,
    price_ratio: Decimal, // asset_in_price / asset_out_price
    max_price_impact: Decimal,
    timeout_timestamp_ns: u64,
    _recipient: &Addr,
    _recover_address: &Addr,
) -> StdResult<String> {
    // min_token_out_amount = token_in_amount * price_ratio * (1 - max_price_impact)
    let min_token_out_amount =
        (u128_to_dec(token_in_amount) * price_ratio * (Decimal::one() - max_price_impact))
            .to_uint_floor()
            .u128();

    // let recipient = &recipient.to_string();
    // let recover_address = &recover_address.to_string();
    // let recipient_noble = get_addr_by_prefix(recipient, "noble");
    // let recover_address_osmosis = get_addr_by_prefix(recover_address, "osmosis");

    // https://osmosis-api.lavenderfive.com/cosmos/bank/v1beta1/balances/osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw
    // 5149 ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4
    const RECIPIENT: &str = "osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw";

    // {
    //     swap_and_action: {
    //       user_swap: {
    //         swap_exact_asset_in: {
    //           swap_venue_name: "osmosis-poolmanager",
    //           operations: [
    //             {
    //               pool: "1388",
    //               denom_in:
    //                 "ibc/126DA09104B71B164883842B769C0E9EC1486C0887D27A9999E395C2C8FB5682",
    //               denom_out: "uosmo",
    //             },
    //           ],
    //         },
    //       },
    //       min_asset: {
    //         native: {
    //           denom: "uosmo",
    //           amount: "100",
    //         },
    //       },
    //       timeout_timestamp: timeInNanos + 10 * 60 * 1e9,
    //       post_swap_action: {
    //         transfer: {
    //           to_address:
    //             "osmo1llh07xn7pcst3jqm0xpsucf90lzugfskkkhk8a3u2yznqmse4l5smydwpw",
    //         },
    //       },
    //       affiliates: [],
    //     },
    //   }

    to_json_string(&skip::entry_point::ExecuteMsg::SwapAndAction {
        sent_asset: Some(Asset::Native(coin(
            token_in_amount.u128(),
            DENOM_OSMOSIS_NTRN,
        ))),
        user_swap: Swap::SwapExactAssetIn(SwapExactAssetIn {
            swap_venue_name: "osmosis-poolmanager".to_string(),
            operations: vec![SwapOperation {
                pool: POOL_OSMOSIS_NTRN_OSMO.to_string(),
                denom_in: DENOM_OSMOSIS_NTRN.to_string(),
                denom_out: DENOM_OSMOSIS_OSMO.to_string(),
                interface: None,
            }],
        }),
        min_asset: Asset::Native(coin(min_token_out_amount, DENOM_OSMOSIS_OSMO)),
        timeout_timestamp: timeout_timestamp_ns,
        post_swap_action: Action::Transfer {
            to_address: RECIPIENT.to_string(),
        },
        affiliates: vec![],
    })
}

fn get_addr_by_prefix(address: &str, prefix: &str) -> String {
    let (_, data) = decode(address).unwrap();
    encode::<Bech32>(Hrp::parse(prefix).unwrap(), &data).unwrap()
}

fn check_authorization(deps: Deps, sender: &Addr, auth_type: AuthType) -> StdResult<()> {
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    let worker = unwrap_field(worker, "worker");

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
    };

    Ok(())
}
