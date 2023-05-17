use bignumber::Decimal256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::Item;
use haloswap::asset::PairInfoRaw;

#[cw_serde]
pub struct Config {
    pub halo_factory: CanonicalAddr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const PAIR_INFO: Item<PairInfoRaw> = Item::new("pair_info");

// Store commission rate for the pair
pub const COMMISSION_RATE_INFO: Item<Decimal256> = Item::new("commission_rate_info");
