use crate::assert::{assert_max_spread, assert_slippage_tolerance};
use crate::state::{Config, COMMISSION_RATE_INFO, CONFIG, PAIR_INFO};

use bignumber::{Decimal256, Uint256};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw_utils::parse_reply_instantiate_data;
use haloswap::asset::{
    Asset, AssetInfo, AssetInfoRaw, PairInfo, PairInfoRaw, LP_TOKEN_RESERVED_AMOUNT,
};
use haloswap::error::ContractError;
use haloswap::formulas::{calculate_lp_token_amount_to_user, compute_offer_amount, compute_swap};
use haloswap::pair::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, PoolResponse, QueryMsg,
    ReverseSimulationResponse, SimulationResponse,
};
use haloswap::querier::query_token_info;
use haloswap::token::InstantiateMsg as TokenInstantiateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:halo-pair";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let pair_info: &PairInfoRaw = &PairInfoRaw {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        liquidity_token: CanonicalAddr::from(vec![]),
        asset_infos: [
            msg.asset_infos[0].to_raw(deps.api)?,
            msg.asset_infos[1].to_raw(deps.api)?,
        ],
        asset_decimals: msg.asset_decimals,
        requirements: msg.requirements,
        commission_rate: msg.commission_rate,
    };

    // Store factory contract address which is used to create pair contract
    CONFIG.save(
        deps.storage,
        &Config {
            halo_factory: info.sender,
        },
    )?;

    PAIR_INFO.save(deps.storage, pair_info)?;

    COMMISSION_RATE_INFO.save(deps.storage, &msg.commission_rate)?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: msg.lp_token_info.clone().lp_token_name,
                symbol: msg.lp_token_info.lp_token_symbol,
                decimals: msg.lp_token_info.lp_token_decimals.unwrap_or(6),
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
            funds: vec![],
            label: "lp".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity {
            assets,
            slippage_tolerance,
            receiver,
        } => provide_liquidity(deps, env, info, assets, slippage_tolerance, receiver),
        ExecuteMsg::Swap {
            offer_asset,
            belief_price,
            max_spread,
            to,
        } => {
            if !offer_asset.is_native_token() {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = if let Some(to_addr) = to {
                Some(deps.api.addr_validate(&to_addr)?)
            } else {
                None
            };
            swap(
                deps,
                env,
                info.clone(),
                info.sender,
                offer_asset,
                belief_price,
                max_spread,
                to_addr,
            )
        }
        ExecuteMsg::UpdateNativeTokenDecimals {
            denom,
            asset_decimals,
        } => update_native_token_decimals(deps, env, info, denom, asset_decimals),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // let contract_addr = info.sender.clone();

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Swap {
            offer_asset,
            belief_price,
            max_spread,
            to,
        }) => {
            // verify amount of asset info is same as the amount in cw20_msg
            if offer_asset.amount != cw20_msg.amount {
                return Err(ContractError::AssetMismatch {});
            }

            // only asset contract can execute this message
            let mut authorized: bool = false;
            let config: PairInfoRaw = PAIR_INFO.load(deps.storage)?;
            let pools: [Asset; 2] =
                config.query_pools(&deps.querier, deps.api, env.contract.address.clone())?;
            for pool in pools.iter() {
                if let AssetInfo::Token { contract_addr, .. } = &pool.info {
                    if contract_addr == &info.sender {
                        authorized = true;
                    }
                }
            }

            if !authorized {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = if let Some(to_addr) = to {
                Some(deps.api.addr_validate(to_addr.as_str())?)
            } else {
                None
            };

            swap(
                deps,
                env,
                info,
                Addr::unchecked(cw20_msg.sender),
                offer_asset,
                belief_price,
                max_spread,
                to_addr,
            )
        }
        Ok(Cw20HookMsg::WithdrawLiquidity {}) => {
            let config: PairInfoRaw = PAIR_INFO.load(deps.storage)?;
            if deps.api.addr_canonicalize(info.sender.as_str())? != config.liquidity_token {
                return Err(ContractError::Unauthorized {});
            }

            let sender_addr = deps.api.addr_validate(cw20_msg.sender.as_str())?;
            withdraw_liquidity(deps, env, info, sender_addr, cw20_msg.amount)
        }
        Err(err) => Err(ContractError::Std(err)),
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let res = parse_reply_instantiate_data(msg).unwrap();
    let liquidity_token = res.contract_address;

    let api = deps.api;
    PAIR_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.liquidity_token = api.addr_canonicalize(&liquidity_token)?;
        Ok(meta)
    })?;

    Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token))
}

/// CONTRACT - should approve contract to use the amount of token
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: [Asset; 2],
    slippage_tolerance: Option<Decimal>,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    for asset in assets.iter() {
        // check the balance of native token is sent with the message
        asset.assert_sent_native_token_balance(&info)?;
    }

    // get information of the pair
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;

    // query the information of the pair of assets
    let mut pools: [Asset; 2] =
        pair_info.query_pools(&deps.querier, deps.api, env.contract.address.clone())?;

    // get the amount of assets that user deposited after checking the assets is same as the assets in pair
    let deposits: [Uint128; 2] = [
        assets
            .iter()
            .find(|a| a.info.equal(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        assets
            .iter()
            .find(|a| a.info.equal(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    // For accurately calculate return values of compute_swap function,
    // It is required to use Decimal256 type for all division calculations
    // So the maximum value when offer_pool and ask_pool are multiplied
    // is the maximum value of Decimal256 which is (2^128 - 1) / 10^18
    // Check the value of offer_pool * ask_pool is overflowed
    let pool_0 = Uint256::from(pools[0].amount);
    let pool_1 = Uint256::from(pools[1].amount);
    let amount_0 = Uint256::from(deposits[0]);
    let amount_1 = Uint256::from(deposits[1]);

    Decimal256::from_uint256((pool_0 + amount_0) * (pool_1 + amount_1));

    // If the asset is a token, the value of pools[i] is correct. But we must take the token from the user.
    // If the asset is a native token, the amount of native token is already sent with the message to the pool.
    // So we must subtract that amount of native token from the pools[i].
    // pools[] will be used to calculate the amount of LP token to mint after.
    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, pool) in pools.iter_mut().enumerate() {
        // If the asset 'pool' is a token, then we need to execute TransferFrom msg to receive funds
        // User must approve the pool contract to transfer the token before calling this function
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: deposits[i],
                })?,
                funds: vec![],
            }));
        } else {
            // If the asset 'pool' is native token, balance is already increased
            // To calculated properly we should subtract user deposit from the pool
            pool.amount = pool.amount.checked_sub(deposits[i])?;
        }
    }

    // if the user provides the slippage tolerance, we should check it
    assert_slippage_tolerance(&slippage_tolerance, &deposits, &pools)?;

    // get the address of the LP token
    let liquidity_token = deps.api.addr_humanize(&pair_info.liquidity_token)?;

    // get total supply of the LP token
    let total_share = query_token_info(&deps.querier, liquidity_token)?.total_supply;

    // calculate the amount of LP token is minted to the user
    let mut share =
        calculate_lp_token_amount_to_user(&info, &pair_info, total_share, deposits, pools).unwrap();

    // prevent providing free token (one of the deposits is zero)
    if share.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // mint LP token to sender
    // if the user provides the receiver, mint LP token to the receiver else mint to the sender
    let receiver = receiver.unwrap_or_else(|| info.sender.to_string());

    if total_share == Uint128::zero() {
        // mint amount of 'share' LP token to the receiver
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&pair_info.liquidity_token)?
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: deps
                    .api
                    .addr_humanize(&pair_info.liquidity_token)?
                    .to_string(),
                amount: Uint128::from(LP_TOKEN_RESERVED_AMOUNT),
            })?,
            funds: vec![],
        }));
        share = share.checked_sub(Uint128::from(LP_TOKEN_RESERVED_AMOUNT))?;
    }

    // mint amount of 'share' LP token to the receiver
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_humanize(&pair_info.liquidity_token)?
            .to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: receiver.to_string(),
            amount: share,
        })?,
        funds: vec![],
    }));

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "provide_liquidity"),
        ("sender", info.sender.as_str()),
        ("receiver", receiver.as_str()),
        ("assets", &format!("{}, {}", assets[0], assets[1])),
        ("share", &share.to_string()),
    ]))
}

pub fn withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;
    let liquidity_addr: Addr = deps.api.addr_humanize(&pair_info.liquidity_token)?;

    let pools: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, env.contract.address)?;
    let total_share: Uint128 = query_token_info(&deps.querier, liquidity_addr)?.total_supply;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    let refund_assets: Vec<Asset> = pools
        .iter()
        .map(|a| Asset {
            info: a.info.clone(),
            amount: a.amount * share_ratio,
        })
        .collect();

    // update pool info
    Ok(Response::new()
        .add_messages(vec![
            refund_assets[0].clone().into_msg(sender.clone())?,
            refund_assets[1].clone().into_msg(sender.clone())?,
            // burn liquidity token
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_humanize(&pair_info.liquidity_token)?
                    .to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            }),
        ])
        .add_attributes(vec![
            ("action", "withdraw_liquidity"),
            ("sender", sender.as_str()),
            ("withdrawn_share", &amount.to_string()),
            (
                "refund_assets",
                &format!("{}, {}", refund_assets[0], refund_assets[1]),
            ),
        ]))
}

// CONTRACT - a user must do token approval
#[allow(clippy::too_many_arguments)]
pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    offer_asset: Asset,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    to: Option<Addr>,
) -> Result<Response, ContractError> {
    offer_asset.assert_sent_native_token_balance(&info)?;

    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;
    // get pool info of the pair contract
    let pools: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, env.contract.address)?;
    // Commission rate OR Fee amount for framework
    let commission_rate = COMMISSION_RATE_INFO.load(deps.storage)?;

    let offer_pool: Asset;
    let ask_pool: Asset;

    let offer_decimal: u8;
    let ask_decimal: u8;

    // If the asset balance is already increased
    // To calculated properly we should subtract user deposit from the pool
    if offer_asset.info.equal(&pools[0].info) {
        offer_pool = Asset::new(
            pools[0].info.clone(),
            pools[0].amount.checked_sub(offer_asset.amount)?,
        );
        ask_pool = pools[1].clone();

        offer_decimal = pair_info.asset_decimals[0];
        ask_decimal = pair_info.asset_decimals[1];
    } else if offer_asset.info.equal(&pools[1].info) {
        offer_pool = Asset::new(
            pools[1].info.clone(),
            pools[1].amount.checked_sub(offer_asset.amount)?,
        );
        ask_pool = pools[0].clone();

        offer_decimal = pair_info.asset_decimals[1];
        ask_decimal = pair_info.asset_decimals[0];
    } else {
        return Err(ContractError::AssetMismatch {});
    }

    let offer_amount = offer_asset.amount;
    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool.amount,
        ask_pool.amount,
        offer_amount,
        commission_rate,
    );

    let return_asset = Asset {
        info: ask_pool.info.clone(),
        amount: return_amount,
    };

    // check max spread limit if exist
    assert_max_spread(
        belief_price,
        max_spread,
        offer_asset.clone(),
        return_asset.clone(),
        spread_amount,
        offer_decimal,
        ask_decimal,
    )?;

    let receiver = to.unwrap_or_else(|| sender.clone());

    let mut messages: Vec<CosmosMsg> = vec![];
    if !return_amount.is_zero() {
        messages.push(return_asset.into_msg(receiver.clone())?);
    }

    // 1. send collateral token from the contract to a user
    // 2. send inactive commission to collector
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "swap"),
        ("sender", sender.as_str()),
        ("receiver", receiver.as_str()),
        ("offer_asset", &offer_asset.info.to_string()),
        ("ask_asset", &ask_pool.info.to_string()),
        ("offer_amount", &offer_amount.to_string()),
        ("return_amount", &return_amount.to_string()),
        ("spread_amount", &spread_amount.to_string()),
        ("commission_amount", &commission_amount.to_string()),
    ]))
}

pub fn update_native_token_decimals(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
    asset_decimals: [u8; 2],
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // permission check
    if info.sender.as_str() != config.halo_factory {
        return Err(ContractError::Unauthorized {});
    }

    let mut pair_info_raw: PairInfoRaw = PAIR_INFO.load(deps.storage)?;

    let mut asset_infos = pair_info_raw.asset_infos;
    for asset_info in asset_infos.iter_mut() {
        if let AssetInfoRaw::NativeToken { denom: d, .. } = asset_info {
            if d == &denom {
                pair_info_raw.asset_decimals = asset_decimals;
            }
        }
    }

    PAIR_INFO.save(
        deps.storage,
        &PairInfoRaw {
            asset_infos,
            ..pair_info_raw
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "update_native_token_decimals"),
        (
            "pair_decimals",
            &format!("{}-{}", asset_decimals[0], asset_decimals[1]),
        ),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Pair {} => Ok(to_binary(&query_pair_info(deps)?)?),
        QueryMsg::Pool {} => Ok(to_binary(&query_pool(deps)?)?),
        QueryMsg::Simulation { offer_asset } => {
            Ok(to_binary(&query_simulation(deps, offer_asset)?)?)
        }
        QueryMsg::ReverseSimulation { ask_asset } => {
            Ok(to_binary(&query_reverse_simulation(deps, ask_asset)?)?)
        }
    }
}

pub fn query_pair_info(deps: Deps) -> Result<PairInfo, ContractError> {
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;
    let pair_info = pair_info.to_normal(deps.api)?;

    Ok(pair_info)
}

pub fn query_pool(deps: Deps) -> Result<PoolResponse, ContractError> {
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;
    let contract_addr = deps.api.addr_humanize(&pair_info.contract_addr)?;
    let assets: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, contract_addr)?;
    let total_share: Uint128 = query_token_info(
        &deps.querier,
        deps.api.addr_humanize(&pair_info.liquidity_token)?,
    )?
    .total_supply;

    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

pub fn query_simulation(
    deps: Deps,
    offer_asset: Asset,
) -> Result<SimulationResponse, ContractError> {
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;

    let contract_addr = deps.api.addr_humanize(&pair_info.contract_addr)?;
    // get pool info of the pair contract
    let pools: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, contract_addr)?;
    // Commission rate OR Fee amount for framework
    let commission_rate = COMMISSION_RATE_INFO.load(deps.storage)?;

    let offer_pool: Asset;
    let ask_pool: Asset;
    if offer_asset.info.equal(&pools[0].info) {
        offer_pool = pools[0].clone();
        ask_pool = pools[1].clone();
    } else if offer_asset.info.equal(&pools[1].info) {
        offer_pool = pools[1].clone();
        ask_pool = pools[0].clone();
    } else {
        return Err(ContractError::AssetMismatch {});
    }

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool.amount,
        ask_pool.amount,
        offer_asset.amount,
        commission_rate,
    );

    Ok(SimulationResponse {
        return_amount,
        spread_amount,
        commission_amount,
    })
}

pub fn query_reverse_simulation(
    deps: Deps,
    ask_asset: Asset,
) -> Result<ReverseSimulationResponse, ContractError> {
    // get pair info
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;

    // get address of the pair contract
    let contract_addr = deps.api.addr_humanize(&pair_info.contract_addr)?;

    // get pool info of the pair contract
    let pools: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, contract_addr)?;
    // Commission rate OR Fee amount for framework
    let commission_rate = COMMISSION_RATE_INFO.load(deps.storage)?;

    let offer_pool: Asset;
    let ask_pool: Asset;
    if ask_asset.info.equal(&pools[0].info) {
        ask_pool = pools[0].clone();
        offer_pool = pools[1].clone();
    } else if ask_asset.info.equal(&pools[1].info) {
        ask_pool = pools[1].clone();
        offer_pool = pools[0].clone();
    } else {
        return Err(ContractError::AssetMismatch {});
    }

    // compute offer amount, spread amount, commission amount when user provide ask amount
    let (offer_amount, spread_amount, commission_amount) = compute_offer_amount(
        offer_pool.amount,
        ask_pool.amount,
        ask_asset.amount,
        commission_rate,
    );

    Ok(ReverseSimulationResponse {
        offer_amount,
        spread_amount,
        commission_amount,
    })
}

// pub fn amount_of(coins: &[Coin], denom: String) -> Uint128 {
//     match coins.iter().find(|x| x.denom == denom) {
//         Some(coin) => coin.amount,
//         None => Uint128::zero(),
//     }
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
