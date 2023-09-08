use bignumber::Decimal256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use haloswap::asset::{LPTokenInfo, AssetInfo};

use crate::state::{CreateStablePoolRequirements, StablePoolInfo};

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
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StablePoolInfo)]
    StablePool {},
}
