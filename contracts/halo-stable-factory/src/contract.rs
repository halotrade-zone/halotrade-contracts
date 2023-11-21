use std::str::FromStr;

use bignumber::Decimal256;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use halo_stable_pair::math::AmpFactor;
use halo_stable_pair::msg::{
    ExecuteMsg as StablePairExecuteMsg, InstantiateMsg as StablePairInstantiateMsg,
};
use halo_stable_pair::state::{
    CreateStablePairRequirements, StablePairInfo, StablePairInfoRaw, StablePairsResponse,
    DEFAULT_COMMISSION_RATE,
};
use haloswap::asset::{AssetInfo, AssetInfoRaw, LPTokenInfo};
use haloswap::factory::NativeTokenDecimalsResponse;
use haloswap::querier::query_balance;

use crate::msg::{ConfigResponse, QueryMsg};
use crate::query::{query_decimals, query_stable_pair_info_from_stable_pairs};
use crate::state::{add_allow_native_token, read_stable_pairs, ALLOW_NATIVE_TOKENS, STABLE_PAIRS};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg},
    state::{pair_key, Config, TmpStablePairInfo, CONFIG, TMP_STABLE_PAIR_INFO},
};

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
        owner: deps.api.addr_validate(info.sender.as_str())?,
        token_code_id: msg.token_code_id,
        stable_pair_code_id: msg.stable_pair_code_id,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateStablePair {
            asset_infos,
            requirements,
            commission_rate,
            lp_token_info,
            amp_factor_info,
        } => execute_create_stable_pair(
            deps,
            env,
            info,
            asset_infos,
            requirements,
            commission_rate,
            lp_token_info,
            amp_factor_info,
        ),
        ExecuteMsg::AddNativeTokenDecimals { denom, decimals } => {
            execute_add_native_token_decimals(deps, env, info, denom, decimals)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_create_stable_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_infos: Vec<AssetInfo>,
    requirements: CreateStablePairRequirements,
    commission_rate: Option<Decimal256>,
    lp_token_info: LPTokenInfo,
    amp_factor_info: AmpFactor,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    // Permission check
    if deps.api.addr_validate(info.sender.as_str())? != config.owner {
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
        match query_decimals(
            asset_info.clone(),
            env.contract.address.clone(),
            &deps.querier,
        ) {
            Ok(decimal) => {
                asset_decimals.push(decimal);
                raw_infos.push(asset_info.to_raw(deps.api)?);
            }
            Err(_) => return Err(StdError::generic_err("asset is invalid")),
        }
    }

    // Get pair key
    let pair_key = pair_key(&raw_infos);

    TMP_STABLE_PAIR_INFO.save(
        deps.storage,
        &TmpStablePairInfo {
            pair_key,
            asset_infos: raw_infos,
            asset_decimals: asset_decimals.clone(),
        },
    )?;
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_stable_pair"),
            (
                "stable_assets",
                &asset_infos
                    .iter()
                    .map(|asset_info| asset_info.to_string()) // Loop and add all asset_info to get stable assets
                    .collect::<Vec<String>>()
                    .join(","),
            ),
        ])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: config.stable_pair_code_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "stable_pair".to_string(),
                msg: to_binary(&StablePairInstantiateMsg {
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
                    amp_factor_info,
                })?,
            }),
            reply_on: ReplyOn::Success,
        }))
}

pub fn execute_add_native_token_decimals(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    decimals: u8,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut res: Response = Response::new();
    let is_native_exist: bool = ALLOW_NATIVE_TOKENS
        .may_load(deps.storage, denom.as_bytes())?
        .is_some();

    // permission check
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let balance = query_balance(&deps.querier, env.contract.address, denom.to_string())?;
    if balance.is_zero() {
        return Err(StdError::generic_err(
            "a balance greater than zero is required by the stable factory for verification",
        ));
    }

    // Add the native token decimals to the allow list
    add_allow_native_token(deps.storage, denom.to_string(), decimals)?;

    // Update the native token decimals for the existing pairs
    let pair_infos = read_stable_pairs(deps.storage, deps.api, None, None)?;

    // If the native token is already exist, then update the decimals for the existing pairs
    if is_native_exist {
        // Messages to update the native token decimals for the existing pairs
        let mut messages: Vec<CosmosMsg> = vec![];

        for pair_info in pair_infos {
            // Get the pair key from the pair info
            let pair_key = pair_key(
                &pair_info
                    .asset_infos
                    .iter()
                    .map(|asset_info| asset_info.to_raw(deps.api).unwrap())
                    .collect::<Vec<AssetInfoRaw>>(),
            );

            // Get raw pair info from the pair key
            let stable_pair_info_raw: StablePairInfoRaw =
                STABLE_PAIRS.load(deps.storage, &pair_key)?;

            // Loop pair_info to identify the native token
            for (i, asset_info) in pair_info.asset_infos.iter().enumerate() {
                // If the asset info is native token, then update the decimals
                if asset_info.is_native_token()
                    && pair_info.asset_infos[i]
                        .query_denom_of_native_token()
                        .unwrap()
                        == denom
                {
                    // Update the native token decimals
                    let mut asset_decimals = stable_pair_info_raw.asset_decimals.clone();

                    asset_decimals[i] = decimals;

                    // Update the stable pair info
                    STABLE_PAIRS.save(
                        deps.storage,
                        &pair_key,
                        &StablePairInfoRaw {
                            asset_decimals: asset_decimals.clone(),
                            ..stable_pair_info_raw.clone()
                        },
                    )?;

                    // Update the stable pair contract
                    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: pair_info.contract_addr.to_string(),
                        msg: to_binary(&StablePairExecuteMsg::UpdateNativeTokenDecimals {
                            denom: denom.clone(),
                            asset_decimals,
                        })?,
                        funds: vec![],
                    }));
                }
            }
        }
        res = res.add_messages(messages);
    }

    Ok(res.add_attributes(vec![
        ("action", "add_allow_native_token"),
        ("denom", &denom),
        ("decimals", &decimals.to_string()),
    ]))
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let tmp_stable_pair_info = TMP_STABLE_PAIR_INFO.load(deps.storage)?;

    let reply = parse_reply_instantiate_data(msg).unwrap();

    let stable_pair_contract = &reply.contract_address;
    let stable_pair_info = query_stable_pair_info_from_stable_pairs(
        &deps.querier,
        Addr::unchecked(stable_pair_contract),
    )?;

    STABLE_PAIRS.save(
        deps.storage,
        &tmp_stable_pair_info.pair_key,
        &StablePairInfoRaw {
            liquidity_token: deps.api.addr_validate(&stable_pair_info.liquidity_token)?,
            contract_addr: deps.api.addr_validate(stable_pair_contract)?,
            asset_infos: tmp_stable_pair_info.asset_infos,
            asset_decimals: tmp_stable_pair_info.asset_decimals,
            requirements: stable_pair_info.requirements,
            commission_rate: Decimal256::from_str(&stable_pair_info.commission_rate.to_string())
                .unwrap(),
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        ("stable_pair_contract_addr", stable_pair_contract),
        ("liquidity_token_addr", &stable_pair_info.liquidity_token),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::StablePair { asset_infos } => to_binary(&query_stable_pair(deps, asset_infos)?),
        QueryMsg::StablePairs { start_after, limit } => {
            to_binary(&query_stable_pairs(deps, start_after, limit)?)
        }
        QueryMsg::NativeTokenDecimals { denom } => {
            to_binary(&query_native_token_decimal(deps, denom)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_validate(state.owner.as_ref())?.to_string(),
        token_code_id: state.token_code_id,
        stable_pair_code_id: state.stable_pair_code_id,
    };

    Ok(resp)
}

pub fn query_stable_pair(deps: Deps, asset_infos: Vec<AssetInfo>) -> StdResult<StablePairInfo> {
    let stable_pair_key = pair_key(
        &asset_infos
            .iter()
            .map(|asset_info| asset_info.to_raw(deps.api).unwrap())
            .collect::<Vec<AssetInfoRaw>>(),
    );
    let stable_pair_info: StablePairInfoRaw = STABLE_PAIRS.load(deps.storage, &stable_pair_key)?;
    stable_pair_info.to_normal(deps.api)
}

pub fn query_stable_pairs(
    deps: Deps,
    start_after: Option<Vec<AssetInfo>>,
    limit: Option<u32>,
) -> StdResult<StablePairsResponse> {
    let start_after = start_after.map(|start_after| {
        start_after
            .iter()
            .map(|asset_info| asset_info.to_raw(deps.api).unwrap())
            .collect::<Vec<AssetInfoRaw>>()
    });

    let stable_pairs: Vec<StablePairInfo> =
        read_stable_pairs(deps.storage, deps.api, start_after, limit)?;
    let resp = StablePairsResponse {
        pairs: stable_pairs,
    };

    Ok(resp)
}

pub fn query_native_token_decimal(
    deps: Deps,
    denom: String,
) -> StdResult<NativeTokenDecimalsResponse> {
    let decimals = ALLOW_NATIVE_TOKENS.load(deps.storage, denom.as_bytes())?;
    Ok(NativeTokenDecimalsResponse { decimals })
}
