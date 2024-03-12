use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, HexBinary, Timestamp, Uint128};
use cw_storage_plus::{Item, Map, Deque};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
}

#[cw_serde]
pub struct AuragonURI {
    pub white: [String; 7],
    pub blue: [String; 7],
    pub gold: [String; 7],
    pub red: [String; 7],
}

#[cw_serde]
pub struct GemInfo {
    pub nft_id: String,
    pub nft_contract: Addr,
}

#[cw_serde]
pub struct UserInfo {
    pub user_addr: Addr,
    pub gem_base: GemInfo,
    pub gem_materials: Vec<GemInfo>,
    pub shield_id: Option<String>,
}

#[cw_serde]
pub struct RandomJob {
    pub request_forge_hash: String,
}

#[cw_serde]
pub struct RequestForgeGemInfo {
    pub user_addr: Addr,
    pub gem_base: GemInfo,
    pub gem_materials: Vec<GemInfo>,
    pub success_rate: String,
    pub shield_id: Option<String>,
}

#[cw_serde]
pub struct UserFinalGemInfo {
    pub user_addr: Addr,
    pub color: String,
    pub star: u8,
}

#[cw_serde]
#[derive(Default)]
pub struct GemMetadata {
    pub color: String,
    pub star: u8,
}

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    /// This is how much the minter takes as a cut when sold
    /// royalties are owed on this token if it is Some
    pub royalty_percentage: Option<u64>,
    /// The payment address, may be different to or the same
    /// as the minter addr
    /// question: how do we validate this?
    pub royalty_payment_address: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const RANDOM_SEED: Item<String> = Item::new("random seed");

pub const RANDOM_JOBS: Map<String, String> = Map::new("random jobs");
