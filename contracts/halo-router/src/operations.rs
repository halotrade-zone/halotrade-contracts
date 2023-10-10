use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, WasmMsg,
};
use halo_stable_factory::query::query_stable_pool_info;
use halo_stable_pool::state::StablePoolInfo;
use halo_stable_pool::msg::ExecuteMsg as StablePoolExecuteMsg;

use crate::state::{Config, CONFIG, STABLE_FACTORY_CONFIG, StableFactoryConfig};

use cw20::Cw20ExecuteMsg;
use haloswap::asset::{Asset, AssetInfo, PairInfo};
use haloswap::pair::Cw20HookMsg as PairHookMsg;
use haloswap::querier::{query_balance, query_pair_info, query_token_balance};
use haloswap::router::SwapOperation;

/// Execute swap operation
/// swap all offer asset to ask asset
pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation: SwapOperation,
    to: Option<String>,
) -> StdResult<Response> {
    if env.contract.address != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }
    let messages: Vec<CosmosMsg> = match operation {
        SwapOperation::HaloSwap {
            offer_asset_info,
            ask_asset_info,
        } => {
            let config: Config = CONFIG.load(deps.as_ref().storage)?;
            let halo_factory = deps.api.addr_humanize(&config.halo_factory)?;
            let pair_info: PairInfo = query_pair_info(
                &deps.querier,
                halo_factory,
                &[offer_asset_info.clone(), ask_asset_info],
            )?;

            let amount = match offer_asset_info.clone() {
                AssetInfo::NativeToken { denom } => {
                    query_balance(&deps.querier, env.contract.address, denom)?
                }
                AssetInfo::Token { contract_addr } => query_token_balance(
                    &deps.querier,
                    deps.api.addr_validate(contract_addr.as_str())?,
                    env.contract.address,
                )?,
            };
            let offer_asset: Asset = Asset {
                info: offer_asset_info,
                amount,
            };

            vec![asset_into_swap_msg(
                deps.as_ref(),
                Addr::unchecked(pair_info.contract_addr),
                offer_asset,
                None,
                to,
            )?]
        },
        SwapOperation::StableSwap {
            offer_asset_info,
            ask_asset_info,
            asset_infos,
        } => {
            // Get stable pool factory address
            let config: StableFactoryConfig = STABLE_FACTORY_CONFIG.load(deps.as_ref().storage)?;
            let stable_factory = deps.api.addr_humanize(&config.halo_stable_factory)?;

            let stable_pool_info: StablePoolInfo = query_stable_pool_info(
                &deps.querier,
                stable_factory,
                &asset_infos,
            )?;

            let amount = match offer_asset_info.clone() {
                AssetInfo::NativeToken { denom } => {
                    query_balance(&deps.querier, env.contract.address, denom)?
                }
                AssetInfo::Token { contract_addr } => query_token_balance(
                    &deps.querier,
                    deps.api.addr_validate(contract_addr.as_str())?,
                    env.contract.address,
                )?,
            };

            let offer_asset: Asset = Asset {
                info: offer_asset_info,
                amount,
            };

            vec![asset_into_stable_swap_msg(
                deps.as_ref(),
                Addr::unchecked(stable_pool_info.contract_addr),
                offer_asset,
                ask_asset_info,
                None,
                to,
            )?]
        }
    };

    Ok(Response::new().add_messages(messages))
}

pub fn asset_into_swap_msg(
    _deps: Deps,
    pair_contract: Addr,
    offer_asset: Asset,
    max_spread: Option<Decimal>,
    to: Option<String>,
) -> StdResult<CosmosMsg> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_contract.to_string(),
            funds: vec![Coin {
                denom,
                amount: offer_asset.amount,
            }],
            msg: to_binary(&PairHookMsg::Swap {
                offer_asset,
                belief_price: None,
                max_spread,
                to,
            })?,
        })),
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&PairHookMsg::Swap {
                    offer_asset,
                    belief_price: None,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}

pub fn asset_into_stable_swap_msg(
    _deps: Deps,
    stable_pool_contract: Addr,
    offer_asset: Asset,
    ask_asset_info: AssetInfo,
    max_spread: Option<Decimal>,
    to: Option<String>,
) -> StdResult<CosmosMsg> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: stable_pool_contract.to_string(),
            funds: vec![Coin {
                denom,
                amount: offer_asset.amount,
            }],
            msg: to_binary(&StablePoolExecuteMsg::StableSwap {
                offer_asset,
                ask_asset: ask_asset_info,
                belief_price: None,
                max_spread,
                to: Some(Addr::unchecked(to.unwrap())),
            })?,
        })),
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: stable_pool_contract.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&StablePoolExecuteMsg::StableSwap {
                    offer_asset,
                    ask_asset: ask_asset_info,
                    belief_price: None,
                    max_spread,
                    to: Some(Addr::unchecked(to.unwrap())),
                })?,
            })?,
        })),
    }
}
