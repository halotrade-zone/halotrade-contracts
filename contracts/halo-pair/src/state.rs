use bignumber::Decimal256;
use cw_storage_plus::Item;
use haloswap::asset::PairInfoRaw;

pub const PAIR_INFO: Item<PairInfoRaw> = Item::new("pair_info");

// Store commission rate for the pair
pub const COMMISSION_RATE_INFO: Item<Decimal256> = Item::new("commission_rate_info");
