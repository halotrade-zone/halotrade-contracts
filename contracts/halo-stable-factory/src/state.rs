use bignumber::Decimal256;
use cosmwasm_std::{CanonicalAddr, Addr, Uint128};
use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;
use haloswap::asset::AssetInfoRaw;

#[cw_serde]
pub struct Config {
    pub owner: CanonicalAddr,
    pub stable_pool_code_id: u64,
    pub token_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct TmpPairInfo {
    pub pair_key: Vec<u8>,
    pub asset_infos: Vec<AssetInfoRaw>,
    pub asset_decimals: Vec<u8>,
}

pub const TMP_PAIR_INFO: Item<TmpPairInfo> = Item::new("tmp_pair_info");

pub fn pair_key(asset_infos: &Vec<AssetInfoRaw>) -> Vec<u8> {
    let mut asset_infos = asset_infos.to_vec();
    // Initialize return value
    let mut key: Vec<u8> = Vec::new();
    // Loop through all Vec<AssetInfoRaw> and append each AssetInfoRaw's bytes to the key
    for asset_info in asset_infos.iter() {
        key.append(&mut asset_info.as_bytes().to_vec());
    }
    // Return the key
    key
}