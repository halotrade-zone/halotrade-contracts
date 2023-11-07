use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use halo_stable_pair::state::StablePairInfoRaw;
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
