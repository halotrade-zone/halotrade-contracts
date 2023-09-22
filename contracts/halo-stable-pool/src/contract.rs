#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, StdError,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw_utils::parse_reply_instantiate_data;
use haloswap::{token::InstantiateMsg as TokenInstantiateMsg, asset::{AssetInfoRaw, Asset, LP_TOKEN_RESERVED_AMOUNT, AssetInfo}, error::ContractError, querier::{query_token_info, query_token_balance}};

use crate::{msg::{InstantiateMsg, ExecuteMsg, QueryMsg}, state::{StablePoolInfoRaw, CONFIG, Config, STABLE_POOL_INFO, COMMISSION_RATE_INFO, AMP_FACTOR_INFO}, assert::assert_stable_slippage_tolerance, math::AmpFactor};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:halo-stable-pool";
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

    // convert Vec<AssetInfo> to Vec<AssetInfoRaw>
    let asset_infos: Vec<AssetInfoRaw> = msg
        .asset_infos
        .iter()
        .map(|asset_info| asset_info.to_raw(deps.api))
        .collect::<StdResult<Vec<AssetInfoRaw>>>()?;

    let stable_pool_info: &StablePoolInfoRaw = &StablePoolInfoRaw {
        contract_addr: deps.api.addr_validate(env.contract.address.as_str())?,
        liquidity_token: Addr::unchecked(""),
        asset_infos,
        asset_decimals: msg.asset_decimals,
        requirements: msg.requirements,
        commission_rate: msg.commission_rate,
    };

    let amp_factor_info: &AmpFactor = &msg.amp_factor_info;

    // Store factory contract address which is used to create stable pool contract
    CONFIG.save(
        deps.storage,
        &Config {
            halo_stable_factory: info.sender,
        },
    )?;

    STABLE_POOL_INFO.save(deps.storage, stable_pool_info)?;

    AMP_FACTOR_INFO.save(deps.storage, amp_factor_info)?;

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

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let res = parse_reply_instantiate_data(msg).unwrap();
    let liquidity_token = res.contract_address;

    let api = deps.api;
    STABLE_POOL_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.liquidity_token = api.addr_validate(&liquidity_token)?;
        Ok(meta)
    })?;

    Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProvideLiquidity {
            assets,
            slippage_tolerance,
            receiver,
        } => provide_liquidity(deps, env, info, assets, slippage_tolerance, receiver),
        ExecuteMsg::RemoveLiquidityByShare { share, assets_min_amount } => {
            remove_liquidity_by_share(deps, env, info, share, assets_min_amount)
        }
        ExecuteMsg::RemoveLiquidityByToken { token_amounts, max_burn_share } => {
            remove_liquidity_by_token(deps, env, info, token_amounts, max_burn_share)
        }
    }
}

/// CONTRACT - should approve contract to use the amount of token
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<Asset>,
    slippage_tolerance: Option<Decimal>,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    for asset in assets.iter() {
        // check the balance of native token is sent with the message
        asset.assert_sent_native_token_balance(&info)?;
    }

    // get information of the stable pool
    let stable_pool_info: StablePoolInfoRaw = STABLE_POOL_INFO.load(deps.storage)?;
    // get amp factor info
    let amp_factor_info: AmpFactor = AMP_FACTOR_INFO.load(deps.storage)?;
    // query the information of the stable pool of assets
    let mut pools: Vec<Asset> =
        stable_pool_info.query_pools(&deps.querier, deps.api, env.contract.address.clone())?;
    // get the amount of assets that user deposited after checking the assets is same as the assets in stable pool
    let deposits: Vec<Uint128> = pools
        .iter()
        .map(|pool| {
            assets
                .iter()
                .find(|asset| asset.info == pool.info)
                .map(|asset| asset.amount)
                .expect("Wrong asset info is given")
        })
        .collect();

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
    assert_stable_slippage_tolerance(&slippage_tolerance, &deposits, &pools)?;

    // get current total amount of assets in the stable pool
    let old_c_amounts: Vec<Uint128> = pools
        .iter()
        .map(|pool| pool.amount)
        .collect::<Vec<Uint128>>();

    // get the address of the LP token
    let liquidity_token = deps.api.addr_validate(&stable_pool_info.liquidity_token.to_string())?;

    // get total supply of the LP token
    let total_share = query_token_info(&deps.querier, liquidity_token)?.total_supply;

    // calculate the amount of LP token is minted to the user
    let mut share = amp_factor_info.compute_lp_amount_for_deposit(&deposits, &old_c_amounts, total_share, Uint128::zero()).unwrap().0;
    // prevent providing free token (one of the deposits is zero)
    if share.is_zero() {
        return Err(ContractError::Std(StdError::generic_err(
            "Share amount is zero",
        )));
    }

    // mint LP token to sender
    // if the user provides the receiver, mint LP token to the receiver else mint to the sender
    let receiver = receiver.unwrap_or_else(|| info.sender.to_string());

    if total_share == Uint128::zero() {
        // mint amount of 'share' LP token to the receiver
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_validate(&stable_pool_info.liquidity_token.to_string())?
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: deps
                    .api
                    .addr_validate(&stable_pool_info.liquidity_token.to_string())?
                    .to_string(),
                amount: Uint128::from(LP_TOKEN_RESERVED_AMOUNT),
            })?,
            funds: vec![],
        }));
        share = share.checked_sub(Uint128::from(LP_TOKEN_RESERVED_AMOUNT)).unwrap();
    }

    // mint amount of 'share' LP token to the receiver
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_validate(&stable_pool_info.liquidity_token.to_string())?
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

pub fn remove_liquidity_by_share(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    share: Uint128,
    assets_min_amount: Vec<Uint128>,
) -> Result<Response, ContractError> {
    // Get stable pool info
    let stable_pool_info: StablePoolInfoRaw = STABLE_POOL_INFO.load(deps.storage)?;
    // Get sender's LP token balance

    let shares_total_supply = query_token_info(&deps.querier, deps.api.addr_validate(&stable_pool_info.liquidity_token.to_string())?)?.total_supply;

    // Get the amount of assets in the stable pool
    let mut pools: Vec<Asset> = stable_pool_info.query_pools(&deps.querier, deps.api, env.contract.address.clone())?;

    // Get the amount of assets that user will receive after removing liquidity
    let assets_amount: Vec<Uint128> = pools
        .iter()
        .map(|pool| {
            // Return the amount of assets that user will receive
            Ok(pool.amount.checked_mul(share)?.checked_div(shares_total_supply)?)
        })
        .collect::<StdResult<Vec<Uint128>>>()?;

    // Check the amount of assets that user will receive is greater than the minimum amount of assets that user wants to receive
    for (i, asset_amount) in assets_amount.iter().enumerate() {
        if *asset_amount <= assets_min_amount[i] {
            return Err(ContractError::Std(StdError::generic_err(
                "Invalid minimum amount of assets",
            )));
        }
    }

    // Send the amount of assets that user will receive to the sender
    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, pool) in pools.iter_mut().enumerate() {
        // If the asset 'pool' is a token, then we need to execute Transfer msg to send funds
        // If the asset 'pool' is native token, the amount of native token is already sent with the message to the pool.
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: assets_amount[i],
                })?,
                funds: vec![],
            }));
        }
    }

    // Get the address of the LP token
    let liquidity_token = deps.api.addr_validate(&stable_pool_info.liquidity_token.to_string())?;

    // Transfer LP token from sender to contract
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: share,
        })?,
        funds: vec![],
    }));

    // Burn LP token from sender
    let burn_msg = Cw20ExecuteMsg::Burn {
        amount: share,
    };

    // If the amount of LP token is zero, then we don't need to burn LP token
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: liquidity_token.to_string(),
        msg: to_binary(&burn_msg)?,
        funds: vec![],
    }));


    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "remove_liquidity_by_share"),
        ("sender", info.sender.as_str()),
        ("share", &share.to_string()),
        ("assets", &format!("{}, {}", assets_amount[0], assets_amount[1])),
    ]))


}

pub fn remove_liquidity_by_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_amounts: Vec<Uint128>,
    max_burn_share: Option<Uint128>,
) -> Result<Response, ContractError> {
    // Get stable pool info
    let stable_pool_info: StablePoolInfoRaw = STABLE_POOL_INFO.load(deps.storage)?;
    // Get sender's LP token balance
    let sender_share_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_validate(&stable_pool_info.liquidity_token.to_string())?,
        info.sender.clone(),
    )?;
    // Get total supply of the LP token
    let shares_total_supply = query_token_info(&deps.querier, deps.api.addr_validate(&stable_pool_info.liquidity_token.to_string())?)?.total_supply;
    // Get amp factor info
    let amp_factor_info: AmpFactor = AMP_FACTOR_INFO.load(deps.storage)?;
    // Get the amount of assets in the stable pool
    let mut pools: Vec<Asset> = stable_pool_info.query_pools(&deps.querier, deps.api, env.contract.address.clone())?;
    // Get current total amount of assets in the stable pool
    let old_c_amounts: Vec<Uint128> = pools
        .iter()
        .map(|pool| pool.amount)
        .collect::<Vec<Uint128>>();
    // Get the amount of LP token that user will burn
    let share = amp_factor_info.compute_lp_amount_for_withdraw(&token_amounts, &old_c_amounts, shares_total_supply, Uint128::zero()).unwrap().0;
    // Check the amount of LP token that user will burn is less than the amount of LP token that user has
    if share > sender_share_balance {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient LP token balance",
        )));
    }
    // If the user provides the maximum amount of LP token that user will burn, check it
    if let Some(max_burn_share) = max_burn_share {
        if share > max_burn_share {
            return Err(ContractError::Std(StdError::generic_err(
                "Invalid maximum amount of LP token",
            )));
        }
    }

    // Send the amount of assets that user will receive to the sender
    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, pool) in pools.iter_mut().enumerate() {
        // If the asset 'pool' is a token, then we need to execute Transfer msg to send funds
        // If the asset 'pool' is native token, the amount of native token is already sent with the message to the pool.
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: token_amounts[i],
                })?,
                funds: vec![],
            }));
        }
    }

    // Get the address of the LP token
    let liquidity_token = deps.api.addr_validate(&stable_pool_info.liquidity_token.to_string())?;

    // Transfer LP token from sender to contract
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: share,
        })?,
        funds: vec![],
    }));

    // Burn LP token from sender
    let burn_msg = Cw20ExecuteMsg::Burn {
        amount: share,
    };

    // If the amount of LP token is zero, then we don't need to burn LP token
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: liquidity_token.to_string(),
        msg: to_binary(&burn_msg)?,
        funds: vec![],
    }));

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "remove_liquidity_by_token"),
        ("sender", info.sender.as_str()),
        ("share", &share.to_string()),
        ("assets", &format!("{}, {}", token_amounts[0], token_amounts[1])),
    ]))

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::StablePool {} => Ok(to_binary(&query_stable_pool_info(deps)?)?),
    }
}

/// Query stable pool info
pub fn query_stable_pool_info(deps: Deps) -> StdResult<StablePoolInfoRaw> {
    let stable_pool_info: StablePoolInfoRaw = STABLE_POOL_INFO.load(deps.storage)?;
    Ok(stable_pool_info)
}
