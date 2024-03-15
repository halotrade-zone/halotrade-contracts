use crate::msg::QueryMsg as StableFactoryQueryMsg;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use halo_stable_pair::msg::QueryMsg as StablePairQueryMsg;
use halo_stable_pair::state::StablePairInfo;
use haloswap::asset::AssetInfo;
use haloswap::factory::NativeTokenDecimalsResponse;
use haloswap::querier::query_token_info;

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

pub fn query_decimals(
    asset_info: AssetInfo,
    account_addr: Addr,
    querier: &QuerierWrapper,
) -> StdResult<u8> {
    match asset_info {
        AssetInfo::NativeToken { denom } => {
            query_stable_native_decimals(querier, account_addr, denom)
        }
        AssetInfo::Token { contract_addr } => {
            let token_info = query_token_info(querier, Addr::unchecked(contract_addr))?;
            Ok(token_info.decimals)
        }
    }
}

pub fn query_stable_native_decimals(
    querier: &QuerierWrapper,
    factory_contract: Addr,
    denom: String,
) -> StdResult<u8> {
    let res: NativeTokenDecimalsResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: factory_contract.to_string(),
            msg: to_binary(&StableFactoryQueryMsg::NativeTokenDecimals { denom })?,
        }))?;
    Ok(res.decimals)
}
