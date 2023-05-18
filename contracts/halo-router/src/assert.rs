use cosmwasm_std::{Addr, Deps, Env, MessageInfo, Response, StdError, StdResult, Uint128};
use haloswap::asset::AssetInfo;
use haloswap::router::SwapOperation;
use std::collections::HashMap;

pub fn assert_minium_receive(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    asset_info: AssetInfo,
    prev_balance: Uint128,
    minium_receive: Uint128,
    receiver: Addr,
) -> StdResult<Response> {
    if env.contract.address != info.sender {
        return Err(StdError::generic_err(
            "unauthorized: assert_minium_receive only can be called by contract itself",
        ));
    }
    let receiver_balance = asset_info.query_pool(&deps.querier, deps.api, receiver)?;
    let swap_amount = receiver_balance.checked_sub(prev_balance)?;

    if swap_amount < minium_receive {
        return Err(StdError::generic_err(format!(
            "assertion failed; minimum receive amount: {}, swap amount: {}",
            minium_receive, swap_amount
        )));
    }

    Ok(Response::default())
}

pub fn assert_operations(operations: &[SwapOperation]) -> StdResult<()> {
    let mut ask_asset_map: HashMap<String, bool> = HashMap::new();
    for operation in operations.iter() {
        let (offer_asset, ask_asset) = match operation {
            SwapOperation::HaloSwap {
                offer_asset_info,
                ask_asset_info,
            } => (offer_asset_info.clone(), ask_asset_info.clone()),
        };

        ask_asset_map.remove(&offer_asset.to_string());
        ask_asset_map.insert(ask_asset.to_string(), true);
    }

    if ask_asset_map.keys().len() != 1 {
        return Err(StdError::generic_err(
            "invalid operations; multiple output token",
        ));
    }

    Ok(())
}
