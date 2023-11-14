use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use halo_stable_pair::state::{StablePairInfo, StablePairInfoRaw};
use haloswap::asset::AssetInfoRaw;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub stable_pair_code_id: u64,
    pub token_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct TmpStablePairInfo {
    pub pair_key: Vec<u8>,
    pub asset_infos: Vec<AssetInfoRaw>,
    pub asset_decimals: Vec<u8>,
}

pub const TMP_STABLE_PAIR_INFO: Item<TmpStablePairInfo> = Item::new("tmp_stable_pair_info");
pub const STABLE_PAIRS: Map<&[u8], StablePairInfoRaw> = Map::new("stable_pair_info");

pub fn pair_key(asset_infos: &[AssetInfoRaw]) -> Vec<u8> {
    let mut asset_infos = asset_infos.to_vec();
    asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    asset_infos
        .iter()
        .map(|asset_info| asset_info.as_bytes())
        .collect::<Vec<&[u8]>>()
        .concat()
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_stable_pairs(
    storage: &dyn Storage,
    api: &dyn Api,
    start_after: Option<Vec<AssetInfoRaw>>,
    limit: Option<u32>,
) -> StdResult<Vec<StablePairInfo>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after).map(Bound::ExclusiveRaw);

    STABLE_PAIRS
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            v.to_normal(api)
        })
        .collect::<StdResult<Vec<StablePairInfo>>>()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<Vec<AssetInfoRaw>>) -> Option<Vec<u8>> {
    start_after.map(|asset_infos| {
        let mut asset_infos = asset_infos.to_vec();
        asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut v = asset_infos
            .iter()
            .map(|asset_info| asset_info.as_bytes())
            .collect::<Vec<&[u8]>>()
            .concat()
            .as_slice()
            .to_vec();
        v.push(1);
        v
    })
}
