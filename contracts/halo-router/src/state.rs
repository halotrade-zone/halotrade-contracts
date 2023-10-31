use bignumber::Decimal256;
use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, CanonicalAddr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub halo_factory: CanonicalAddr,
}
#[cw_serde]
pub struct StableFactoryConfig {
    pub halo_stable_factory: CanonicalAddr,
}

#[cw_serde]
pub struct PlatformInfo {
    pub fee: Decimal256,
    pub manager: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const STABLE_FACTORY_CONFIG: Item<StableFactoryConfig> = Item::new("stable_factory_config");

pub const PLATFORM_INFO: Item<PlatformInfo> = Item::new("platform_info");
