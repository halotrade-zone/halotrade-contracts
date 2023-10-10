use std::str::FromStr;

use bignumber::{Decimal256, Uint256};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Api, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use halo_stable_factory::query::query_stable_pool_info;
use halo_stable_pool::querier::stable_simulate;
use halo_stable_pool::state::StablePoolInfo;

use crate::assert::{assert_minium_receive, assert_operations};
use crate::operations::execute_swap_operation;
use crate::state::{Config, PlatformInfo, CONFIG, PLATFORM_INFO};

use cw20::Cw20ReceiveMsg;
use haloswap::asset::{Asset, AssetInfo, PairInfo};
use haloswap::pair::SimulationResponse;
use haloswap::querier::{
    query_balance, query_pair_info, query_token_balance, reverse_simulate, simulate,
};
use haloswap::router::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    SimulateSwapOperationsResponse, SwapOperation,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:halo-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            halo_factory: deps.api.addr_canonicalize(&msg.halo_factory)?,
        },
    )?;

    PLATFORM_INFO.save(
        deps.storage,
        &PlatformInfo {
            fee: Decimal256::from_str("0.01").unwrap(),
            manager: info.sender,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            to,
        } => {
            let api = deps.api;
            execute_swap_operations(
                deps,
                env,
                info.sender,
                operations,
                minimum_receive,
                optional_addr_validate(api, to)?,
            )
        }
        ExecuteMsg::ExecuteSwapOperation { operation, to } => {
            let api = deps.api;
            execute_swap_operation(
                deps,
                env,
                info,
                operation,
                optional_addr_validate(api, to)?.map(|v| v.to_string()),
            )
        }
        ExecuteMsg::AssertMinimumReceive {
            asset_info,
            prev_balance,
            minimum_receive,
            receiver,
        } => assert_minium_receive(
            deps.as_ref(),
            env,
            info,
            asset_info,
            prev_balance,
            minimum_receive,
            deps.api.addr_validate(&receiver)?,
        ),
        ExecuteMsg::UpdatePlatformFee { fee, manager } => {
            // only manager can update platform fee
            let mut platform_info = PLATFORM_INFO.load(deps.storage)?;
            if platform_info.manager != info.sender {
                return Err(StdError::generic_err("unauthorized"));
            }
            platform_info.fee = fee;
            platform_info.manager = deps.api.addr_validate(&manager)?;
            PLATFORM_INFO.save(deps.storage, &platform_info)?;

            Ok(Response::new().add_attributes([
                ("action", "update_platform_fee"),
                ("fee", &fee.to_string()),
                ("manager", &manager),
            ]))
        }
    }
}

fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<Addr>> {
    let addr = if let Some(addr) = addr {
        Some(api.addr_validate(&addr)?)
    } else {
        None
    };

    Ok(addr)
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let sender = deps.api.addr_validate(&cw20_msg.sender)?;
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            to,
        } => {
            let api = deps.api;
            execute_swap_operations(
                deps,
                env,
                sender,
                operations,
                minimum_receive,
                optional_addr_validate(api, to)?,
            )
        }
    }
}

pub fn execute_swap_operations(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    operations: Vec<SwapOperation>,
    minimum_receive: Option<Uint128>,
    to: Option<Addr>,
) -> StdResult<Response> {
    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(StdError::generic_err("must provide operations"));
    }

    // Assert the operations are properly set
    assert_operations(&operations)?;

    let to = if let Some(to) = to { to } else { sender };
    let target_asset_info = operations.last().unwrap().get_target_asset_info();

    // collect platform fee
    let mut res: Response = Response::new();
    let platform_info = PLATFORM_INFO.load(deps.storage)?;
    if platform_info.fee > Decimal256::zero() {
        let offer_asset_info = operations.first().unwrap().get_offer_asset_info();
        match offer_asset_info {
            AssetInfo::NativeToken { denom } => {
                let offer_balance =
                    query_balance(&deps.querier, env.contract.address.clone(), denom.clone())?;
                let fee_amount = Uint256::from(offer_balance) * platform_info.fee;

                res = res.add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: platform_info.manager.to_string(),
                    amount: coins(fee_amount.into(), denom),
                }));
            }
            AssetInfo::Token { contract_addr } => {
                let offer_balance = query_token_balance(
                    &deps.querier,
                    deps.api.addr_validate(contract_addr.as_str())?,
                    env.contract.address.clone(),
                )
                .unwrap();
                let fee_amount = Uint256::from(offer_balance) * platform_info.fee;
                res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr,
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                        recipient: platform_info.manager.to_string(),
                        amount: fee_amount.into(),
                    })?,
                    funds: vec![],
                }));
            }
        };
    }

    let mut operation_index = 0;
    let mut messages: Vec<CosmosMsg> = operations
        .into_iter()
        .map(|op| {
            operation_index += 1;
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: op,
                    to: if operation_index == operations_len {
                        Some(to.to_string())
                    } else {
                        None
                    },
                })?,
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    // Execute minimum amount assertion
    if let Some(minimum_receive) = minimum_receive {
        let receiver_balance = target_asset_info.query_pool(&deps.querier, deps.api, to.clone())?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::AssertMinimumReceive {
                asset_info: target_asset_info,
                prev_balance: receiver_balance,
                minimum_receive,
                receiver: to.to_string(),
            })?,
        }))
    }

    Ok(res.add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::SimulateSwapOperations {
            offer_amount,
            operations,
        } => to_binary(&simulate_swap_operations(deps, offer_amount, operations)?),
        QueryMsg::ReverseSimulateSwapOperations {
            ask_amount,
            operations,
        } => to_binary(&reverse_simulate_swap_operations(
            deps, ask_amount, operations,
        )?),
        QueryMsg::PlatformFee {} => to_binary(&query_platform_fee(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        halo_factory: deps.api.addr_humanize(&state.halo_factory)?.to_string(),
    };

    Ok(resp)
}

fn simulate_swap_operations(
    deps: Deps,
    offer_amount: Uint128,
    operations: Vec<SwapOperation>,
) -> StdResult<SimulateSwapOperationsResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let halo_factory = deps.api.addr_humanize(&config.halo_factory)?;

    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(StdError::generic_err("must provide operations"));
    }

    let mut offer_amount = offer_amount;

    // load platform info
    let platform_info = PLATFORM_INFO.load(deps.storage)?;
    if platform_info.fee > Decimal256::zero() {
        offer_amount = offer_amount
            .checked_sub((Uint256::from(offer_amount) * platform_info.fee).into())
            .unwrap();
    }

    for operation in operations.into_iter() {
        match operation {
            SwapOperation::HaloSwap {
                offer_asset_info,
                ask_asset_info,
            } => {
                let pair_info: PairInfo = query_pair_info(
                    &deps.querier,
                    halo_factory.clone(),
                    &[offer_asset_info.clone(), ask_asset_info.clone()],
                )?;

                let res: SimulationResponse = simulate(
                    &deps.querier,
                    Addr::unchecked(pair_info.contract_addr),
                    &Asset {
                        info: offer_asset_info,
                        amount: offer_amount,
                    },
                )?;
                offer_amount = res.return_amount;
            },
            SwapOperation::StableSwap {
                offer_asset_info,
                ask_asset_info,
                asset_infos
            } => {
                let stable_pool_info: StablePoolInfo = query_stable_pool_info(
                    &deps.querier,
                    halo_factory.clone(),
                    &asset_infos,
                )?;

                let res: SimulationResponse = stable_simulate(
                    &deps.querier,
                    Addr::unchecked(stable_pool_info.contract_addr),
                    &Asset {
                        info: offer_asset_info,
                        amount: offer_amount,
                    },
                    &ask_asset_info,
                )?;
                offer_amount = res.return_amount;
            }
        }
    }

    Ok(SimulateSwapOperationsResponse {
        amount: offer_amount,
    })
}

fn reverse_simulate_swap_operations(
    deps: Deps,
    ask_amount: Uint128,
    operations: Vec<SwapOperation>,
) -> StdResult<SimulateSwapOperationsResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(StdError::generic_err("must provide operations"));
    }

    let mut ask_amount = ask_amount;
    for operation in operations.into_iter().rev() {
        ask_amount = match operation {
            SwapOperation::HaloSwap {
                offer_asset_info,
                ask_asset_info,
            } => {
                let halo_factory = deps.api.addr_humanize(&config.halo_factory)?;

                reverse_simulate_return_amount(
                    deps,
                    halo_factory,
                    ask_amount,
                    offer_asset_info,
                    ask_asset_info,
                )
                .unwrap()
            }
            SwapOperation::StableSwap {
                offer_asset_info,
                ask_asset_info,
                asset_infos
            } => {
                Uint128::zero()
            }
        }
    }

    Ok(SimulateSwapOperationsResponse { amount: ask_amount })
}

pub fn query_platform_fee(deps: Deps) -> StdResult<Decimal256> {
    let platform_info = PLATFORM_INFO.load(deps.storage)?;
    Ok(platform_info.fee)
}

fn reverse_simulate_return_amount(
    deps: Deps,
    factory: Addr,
    ask_amount: Uint128,
    offer_asset_info: AssetInfo,
    ask_asset_info: AssetInfo,
) -> StdResult<Uint128> {
    let pair_info: PairInfo = query_pair_info(
        &deps.querier,
        factory,
        &[offer_asset_info, ask_asset_info.clone()],
    )?;

    let res: haloswap::pair::ReverseSimulationResponse = reverse_simulate(
        &deps.querier,
        Addr::unchecked(pair_info.contract_addr),
        &Asset {
            amount: ask_amount,
            info: ask_asset_info,
        },
    )?;

    // load platform info
    let platform_info = PLATFORM_INFO.load(deps.storage)?;
    if platform_info.fee > Decimal256::zero() {
        Ok(res
            .offer_amount
            .checked_add((Uint256::from(res.offer_amount) * platform_info.fee).into())
            .unwrap())
    } else {
        Ok(res.offer_amount)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
