use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, CanonicalAddr};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub halo_factory: CanonicalAddr,
}

#[cw_serde]
pub struct PlatformInfo {
    pub fee: u64,
    pub collector: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const PLATFORM_INFO: Item<PlatformInfo> = Item::new("platform_info");
