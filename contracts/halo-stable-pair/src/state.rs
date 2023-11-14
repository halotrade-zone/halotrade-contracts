use bignumber::Decimal256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, QuerierWrapper, StdResult, Uint128};
use cw_storage_plus::Item;
use haloswap::asset::{Asset, AssetInfo, AssetInfoRaw};

use crate::math::AmpFactor;

#[cw_serde]
pub struct Config {
    pub halo_stable_factory: Addr,
}
/// Stable pair config
pub const CONFIG: Item<Config> = Item::new("config");
/// Stable pair info
pub const STABLE_PAIR_INFO: Item<StablePairInfoRaw> = Item::new("pair_info");
/// Default commission rate == 0.3%
pub const DEFAULT_COMMISSION_RATE: &str = "0.003";
// Store commission rate for the pair
pub const COMMISSION_RATE_INFO: Item<Decimal256> = Item::new("commission_rate_info");
// Amplification factor info
pub const AMP_FACTOR_INFO: Item<AmpFactor> = Item::new("amp_factor_info");

// We define a custom struct for each query response
#[cw_serde]
pub struct StablePairInfo {
    pub asset_infos: Vec<AssetInfo>,
    pub contract_addr: String,
    pub liquidity_token: String,
    pub asset_decimals: Vec<u8>,
    pub requirements: CreateStablePairRequirements,
    pub commission_rate: Decimal256,
}

#[cw_serde]
pub struct StablePairsResponse {
    pub pairs: Vec<StablePairInfo>,
}

#[cw_serde]
pub struct StablePairInfoRaw {
    pub asset_infos: Vec<AssetInfoRaw>,
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
    pub asset_decimals: Vec<u8>,
    pub requirements: CreateStablePairRequirements,
    pub commission_rate: Decimal256,
}

impl StablePairInfoRaw {
    pub fn to_normal(&self, api: &dyn Api) -> StdResult<StablePairInfo> {
        Ok(StablePairInfo {
            asset_infos: self
                .asset_infos
                .iter()
                .map(|x| x.to_normal(api))
                .collect::<StdResult<Vec<AssetInfo>>>()?,
            contract_addr: api.addr_validate(self.contract_addr.as_ref())?.to_string(),
            liquidity_token: api
                .addr_validate(self.liquidity_token.as_ref())?
                .to_string(),
            asset_decimals: self.asset_decimals.clone(),
            requirements: self.requirements.clone(),
            commission_rate: self.commission_rate,
        })
    }

    pub fn query_pairs(
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
pub struct CreateStablePairRequirements {
    pub whitelist: Vec<Addr>,
    pub asset_minimum: Vec<Uint128>,
}

pub fn decrease_decimals(amount: Uint128, decimals: u8) -> Uint128 {
    let mut amount = amount;
    for _ in 0..decimals - 6 {
        amount /= Uint128::from(10u128);
    }
    amount
}

pub fn increase_decimals(amount: Uint128, decimals: u8) -> Uint128 {
    let mut amount = amount;
    for _ in 0..decimals - 6 {
        amount *= Uint128::from(10u128);
    }
    amount
}
