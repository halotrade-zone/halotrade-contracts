use std::str::FromStr;

use bignumber::Decimal256;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, StdError, SubMsg, CosmosMsg, WasmMsg, attr, to_binary, ReplyOn};
use cw2::set_contract_version;
use halo_stable_pool::state::{CreateStablePoolRequirements, DEFAULT_COMMISSION_RATE};
use halo_stable_pool::msg::InstantiateMsg as StablePoolInstantiateMsg;
use haloswap::asset::{AssetInfo, LPTokenInfo, AssetInfoRaw};

use crate::{msg::{InstantiateMsg, ExecuteMsg}, state::{Config, CONFIG, pair_key, TMP_PAIR_INFO, TmpPairInfo}};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:halo-stable-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        token_code_id: msg.token_code_id,
        stable_pool_code_id: msg.stable_pool_code_id,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateStablePool {
            asset_infos,
            requirements,
            commission_rate,
            lp_token_info,
        } => execute_create_stable_pool(
            deps,
            env,
            info,
            asset_infos,
            requirements,
            commission_rate,
            lp_token_info,
        ),
    }
}

pub fn execute_create_stable_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_infos: Vec<AssetInfo>,
    requirements: CreateStablePoolRequirements,
    commission_rate: Option<Decimal256>,
    lp_token_info: LPTokenInfo,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    // Permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Don't allow to create with same asset in Vec<AssetInfo> by loop and compare
    for (i, asset_info) in asset_infos.iter().enumerate() {
        for (j, asset_info2) in asset_infos.iter().enumerate() {
            if i != j && asset_info == asset_info2 {
                return Err(StdError::generic_err("Cannot create with same asset"));
            }
        }
    }

    // Commission rate must be between 0 and 1 equivalents to 0% to 100%
    if let Some(commission_rate) = commission_rate {
        if commission_rate > Decimal256::one() {
            return Err(StdError::generic_err(
                "commission rate must be between 0 and 1 (equivalents to 0% to 100%)",
            ));
        }
    }

    // Instantiate asset decimals vec
    let mut asset_decimals: Vec<u8> = Vec::new();
    // Instantiate raw asset infos vec
    let mut raw_infos: Vec<AssetInfoRaw> = Vec::new();

    // Loop and check all asset decimals
    for asset_info in asset_infos.iter() {
        match asset_info.query_decimals(env.contract.address.clone(), &deps.querier) {
            Ok(decimal) => {
                asset_decimals.push(decimal);
                raw_infos.push(asset_info.to_raw(deps.api)?);
            },
            Err(_) => return Err(StdError::generic_err("asset is invalid")),
        }
    }

    // Get pair key
    let pair_key = pair_key(&raw_infos);

    TMP_PAIR_INFO.save(
        deps.storage,
        &TmpPairInfo {
            pair_key,
            asset_infos: raw_infos,
            asset_decimals: asset_decimals.clone(),
        },
    )?;
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_stable_pool"),
            ("stable_assets",
            // Loop and add all asset_info to get stable assets
            &asset_infos.iter().map(|asset_info| asset_info.to_string()).collect::<Vec<String>>().join(",")),
        ])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: config.stable_pool_code_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "stable_pool".to_string(),
                msg: to_binary(&StablePoolInstantiateMsg {
                    asset_infos,
                    token_code_id: config.token_code_id,
                    asset_decimals,
                    requirements,
                    commission_rate: commission_rate
                        .unwrap_or_else(|| Decimal256::from_str(DEFAULT_COMMISSION_RATE).unwrap()),
                    lp_token_info: LPTokenInfo {
                        lp_token_name: lp_token_info.lp_token_name,
                        lp_token_symbol: lp_token_info.lp_token_symbol,
                        lp_token_decimals: lp_token_info.lp_token_decimals,
                    },
                })?,
            }),
            reply_on: ReplyOn::Success,
        }))
}