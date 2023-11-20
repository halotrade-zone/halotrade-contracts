use bignumber::Decimal256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use halo_stable_pair::{
    math::AmpFactor,
    state::{CreateStablePairRequirements, StablePairInfo, StablePairsResponse},
};
use haloswap::asset::{AssetInfo, LPTokenInfo};

#[cw_serde]
pub struct InstantiateMsg {
    /// Stable Pair contract code ID, which is used to
    pub stable_pair_code_id: u64,
    pub token_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create Stable Pair
    CreateStablePair {
        /// Stable asset infos
        asset_infos: Vec<AssetInfo>,
        /// The requiments to create a pair
        requirements: CreateStablePairRequirements,
        /// Commission rate for the pair
        commission_rate: Option<Decimal256>,
        /// LP token info
        lp_token_info: LPTokenInfo,
        /// Amplification coefficient for the pair
        amp_factor_info: AmpFactor,
    },
    AddNativeTokenDecimals {
        denom: String,
        decimals: u8,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StablePairInfo)]
    StablePair { asset_infos: Vec<AssetInfo> },
    #[returns(StablePairsResponse)]
    StablePairs {
        start_after: Option<Vec<AssetInfo>>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub stable_pair_code_id: u64,
    pub token_code_id: u64,
}
