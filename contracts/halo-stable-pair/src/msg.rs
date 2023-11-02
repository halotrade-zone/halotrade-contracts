use bignumber::Decimal256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use haloswap::{
    asset::{Asset, AssetInfo, LPTokenInfo},
    pair::SimulationResponse,
};

use crate::{
    math::AmpFactor,
    state::{CreateStablePairRequirements, StablePairInfo},
};

#[cw_serde]
pub struct InstantiateMsg {
    /// Stable asset infos
    pub asset_infos: Vec<AssetInfo>,
    /// Token contract code id for initialization
    pub token_code_id: u64,
    pub asset_decimals: Vec<u8>,
    /// The requiments to the first time provide liquidity
    pub requirements: CreateStablePairRequirements,
    /// Commission rate for the pair
    pub commission_rate: Decimal256,
    /// lp token info
    pub lp_token_info: LPTokenInfo,
    /// Amplification coefficient for the pair
    pub amp_factor_info: AmpFactor,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ProvideLiquidity {
        assets: Vec<Asset>,
        slippage_tolerance: Option<Decimal>,
        receiver: Option<String>,
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
pub enum Cw20StableHookMsg {
    StableSwap {
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<Addr>,
    },
    RemoveLiquidityByShare {
        share: Uint128,
        assets_min_amount: Option<Vec<Uint128>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StablePairInfo)]
    StablePair {},
    #[returns(SimulationResponse)]
    StableSimulation {
        offer_asset: Asset,
        ask_asset: AssetInfo,
    },
    #[returns(Uint128)]
    ProvideLiquiditySimulation { assets: Vec<Asset> },
    #[returns(Vec<Uint128>)]
    RemoveLiquidityByShareSimulation { share: Uint128 },
    #[returns(Uint128)]
    RemoveLiquidityByTokenSimulation { assets: Vec<Asset> },
}
