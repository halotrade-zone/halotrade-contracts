use bignumber::Decimal256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use halo_stable_pool::state::CreateStablePoolRequirements;
use haloswap::asset::{AssetInfo, LPTokenInfo};

#[cw_serde]
pub struct InstantiateMsg {
    /// Stable Pool contract code ID, which is used to
    pub stable_pool_code_id: u64,
    pub token_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create Stable Pool
    CreateStablePool {
        /// Stable asset infos
        asset_infos: Vec<AssetInfo>,
        /// The requiments to create a pair
        requirements: CreateStablePoolRequirements,
        /// Commission rate for the pair
        commission_rate: Option<Decimal256>,
        /// lp token info
        lp_token_info: LPTokenInfo,
    },
}