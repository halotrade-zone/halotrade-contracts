use cosmwasm_std::{QuerierWrapper, Addr, StdResult, QueryRequest, WasmQuery, to_binary};
use haloswap::{asset::{Asset, AssetInfo}, pair::SimulationResponse};
use crate::msg::QueryMsg as StablePoolQueryMsg;

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
