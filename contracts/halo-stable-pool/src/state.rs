use bignumber::Decimal256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, CanonicalAddr, QuerierWrapper, StdResult, Uint128};
use cw_storage_plus::Item;
use haloswap::asset::{Asset, AssetInfo, AssetInfoRaw};

use crate::math::AmpFactor;

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
// Amplification factor info
pub const AMP_FACTOR_INFO: Item<AmpFactor> = Item::new("amp_factor_info");

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
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
    pub asset_decimals: Vec<u8>,
    pub requirements: CreateStablePoolRequirements,
    pub commission_rate: Decimal256,
}

impl StablePoolInfoRaw {
    pub fn to_normal(&self, api: &dyn Api) -> StdResult<StablePoolInfo> {
        Ok(StablePoolInfo {
            asset_infos: self
                .asset_infos
                .iter()
                .map(|x| x.to_normal(api))
                .collect::<StdResult<Vec<AssetInfo>>>()?,
            contract_addr: api
                .addr_validate(&self.contract_addr.to_string())?
                .to_string(),
            liquidity_token: api
                .addr_validate(&self.liquidity_token.to_string())?
                .to_string(),
            asset_decimals: self.asset_decimals.clone(),
            requirements: self.requirements.clone(),
            commission_rate: self.commission_rate,
        })
    }

    pub fn query_pools(
        &self,
        querier: &QuerierWrapper,
        api: &dyn Api,
        contract_addr: Addr,
    ) -> StdResult<Vec<Asset>> {
        let info: Vec<AssetInfo> = self
            .asset_infos
            .iter()
            .map(|x| x.to_normal(api))
            .collect::<StdResult<Vec<AssetInfo>>>()?;
        let mut assets: Vec<Asset> = Vec::new();
        for asset_info in info.iter() {
            let asset = Asset {
                info: asset_info.clone(),
                amount: asset_info.query_pool(querier, api, contract_addr.clone())?,
            };
            assets.push(asset);
        }
        Ok(assets)
    }
}

#[cw_serde]
pub struct CreateStablePoolRequirements {
    pub whitelist: Vec<Addr>,
    pub asset_minimum: Vec<Uint128>,
}
