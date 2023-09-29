use bignumber::Decimal256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128, Addr};
use cw20::Cw20ReceiveMsg;
use haloswap::asset::{LPTokenInfo, AssetInfo, Asset};

use crate::{state::{CreateStablePoolRequirements, StablePoolInfo}, math::AmpFactor};

#[cw_serde]
pub struct InstantiateMsg {
    /// Stable asset infos
    pub asset_infos: Vec<AssetInfo>,
    /// Token contract code id for initialization
    pub token_code_id: u64,
    pub asset_decimals: Vec<u8>,
    /// The requiments to the first time provide liquidity
    pub requirements: CreateStablePoolRequirements,
    /// Commission rate for the pair
    pub commission_rate: Decimal256,
    /// lp token info
    pub lp_token_info: LPTokenInfo,
    /// Amplification coefficient for the pool
    pub amp_factor_info: AmpFactor,
}

#[cw_serde]
pub enum ExecuteMsg {
    ProvideLiquidity {
        assets: Vec<Asset>,
        slippage_tolerance: Option<Decimal>,
        receiver: Option<String>,
    },
    RemoveLiquidityByShare {
        share: Uint128,
        assets_min_amount: Option<Vec<Uint128>>,
    },
    RemoveLiquidityByToken {
        assets: Vec<Asset>,
        max_burn_share: Option<Uint128>,
    },
    StableSwap {
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<Addr>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StablePoolInfo)]
    StablePool {},
}
