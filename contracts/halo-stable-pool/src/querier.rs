use crate::msg::QueryMsg as StablePoolQueryMsg;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use haloswap::{
    asset::{Asset, AssetInfo},
    pair::SimulationResponse,
};

pub fn stable_simulate(
    querier: &QuerierWrapper,
    stable_pool_contract: Addr,
    offer_asset: &Asset,
    ask_asset: &AssetInfo,
) -> StdResult<SimulationResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: stable_pool_contract.to_string(),
        msg: to_binary(&StablePoolQueryMsg::StableSimulation {
            offer_asset: offer_asset.clone(),
            ask_asset: ask_asset.clone(),
        })?,
    }))
}
