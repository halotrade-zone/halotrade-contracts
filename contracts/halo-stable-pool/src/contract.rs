#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use haloswap::{token::InstantiateMsg as TokenInstantiateMsg, asset::AssetInfoRaw};

use crate::{msg::InstantiateMsg, state::{StablePoolInfoRaw, CONFIG, Config, STABLE_POOL_INFO, COMMISSION_RATE_INFO}};

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

    let pair_info: &StablePoolInfoRaw = &StablePoolInfoRaw {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        liquidity_token: CanonicalAddr::from(vec![]),
        asset_infos,
        asset_decimals: msg.asset_decimals,
        requirements: msg.requirements,
        commission_rate: msg.commission_rate,
    };

    // Store factory contract address which is used to create pair contract
    CONFIG.save(
        deps.storage,
        &Config {
            halo_stable_factory: info.sender,
        },
    )?;

    STABLE_POOL_INFO.save(deps.storage, pair_info)?;

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