use cosmwasm_std::{QuerierWrapper, Addr, WasmQuery, StdResult, QueryRequest, to_binary};
use halo_stable_pool::state::StablePoolInfo;
use halo_stable_pool::msg::QueryMsg as StablePoolQueryMsg;

pub fn query_stable_pool_info_from_stable_pools(
    querier: &QuerierWrapper,
    stable_pool_contract: Addr,
) -> StdResult<StablePoolInfo> {
    let stable_pool_info: StablePoolInfo = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: stable_pool_contract.to_string(),
        msg: to_binary(&StablePoolQueryMsg::StablePool {})?,
    }))?;

    Ok(stable_pool_info)
}