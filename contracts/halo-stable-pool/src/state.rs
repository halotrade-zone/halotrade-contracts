use bignumber::Decimal256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, CanonicalAddr};
use cw_storage_plus::Item;
use haloswap::asset::{AssetInfoRaw, AssetInfo};

#[cw_serde]
pub struct Config {
    pub halo_stable_factory: Addr,
}
/// Stable pool config
pub const CONFIG: Item<Config> = Item::new("config");
/// Stable pool info
pub const STABLE_POOL_INFO: Item<StablePoolInfoRaw> = Item::new("pair_info");
/// Default commission rate == 0.3%
pub const DEFAULT_COMMISSION_RATE: &str = "0.003";
// Store commission rate for the pair
pub const COMMISSION_RATE_INFO: Item<Decimal256> = Item::new("commission_rate_info");

// We define a custom struct for each query response
#[cw_serde]
pub struct StablePoolInfo {
    pub asset_infos: Vec<AssetInfo>,
    pub contract_addr: String,
    pub liquidity_token: String,
    pub asset_decimals: Vec<u8>,
    pub requirements: CreateStablePoolRequirements,
    pub commission_rate: Decimal256,
}

#[cw_serde]
pub struct StablePoolInfoRaw {
    pub asset_infos: Vec<AssetInfoRaw>,
    pub contract_addr: CanonicalAddr,
    pub liquidity_token: CanonicalAddr,
    pub asset_decimals: Vec<u8>,
    pub requirements: CreateStablePoolRequirements,
    pub commission_rate: Decimal256,
}

#[cw_serde]
pub struct CreateStablePoolRequirements {
    pub whitelist: Vec<Addr>,
    pub asset_minimum: Vec<Uint128>,
}