use crate::msg::QueryMsg as StableFactoryQueryMsg;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use halo_stable_pool::msg::QueryMsg as StablePoolQueryMsg;
use halo_stable_pool::state::StablePoolInfo;
use haloswap::asset::AssetInfo;

pub fn query_stable_pool_info_from_stable_pools(
    querier: &QuerierWrapper,
    stable_pool_contract: Addr,
) -> StdResult<StablePoolInfo> {
    let stable_pool_info: StablePoolInfo =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: stable_pool_contract.to_string(),
            msg: to_binary(&StablePoolQueryMsg::StablePool {})?,
        }))?;

    Ok(stable_pool_info)
}

pub fn query_stable_pool_info(
    querier: &QuerierWrapper,
    stable_factory_contract: Addr,
    asset_infos: &Vec<AssetInfo>,
) -> StdResult<StablePoolInfo> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: stable_factory_contract.to_string(),
        msg: to_binary(&StableFactoryQueryMsg::StablePool {
            asset_infos: asset_infos.clone(),
        })?,
    }))
}
