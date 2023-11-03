use crate::msg::QueryMsg as StableFactoryQueryMsg;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use halo_stable_pair::msg::QueryMsg as StablePairQueryMsg;
use halo_stable_pair::state::StablePairInfo;
use haloswap::asset::AssetInfo;

pub fn query_stable_pair_info_from_stable_pairs(
    querier: &QuerierWrapper,
    stable_pair_contract: Addr,
) -> StdResult<StablePairInfo> {
    let stable_pair_info: StablePairInfo =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: stable_pair_contract.to_string(),
            msg: to_binary(&StablePairQueryMsg::StablePair {})?,
        }))?;

    Ok(stable_pair_info)
}

pub fn query_stable_pair_info(
    querier: &QuerierWrapper,
    stable_factory_contract: Addr,
    asset_infos: &[AssetInfo],
) -> StdResult<StablePairInfo> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: stable_factory_contract.to_string(),
        msg: to_binary(&StableFactoryQueryMsg::StablePair {
            asset_infos: asset_infos.to_vec(),
        })?,
    }))
}
