use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
    pub contract_owner: Addr,
}

#[cw_serde]
pub struct RandomJob {
    pub request_forge_hash: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const RANDOM_SEED: Item<String> = Item::new("random seed");

pub const RANDOM_JOBS: Map<String, String> = Map::new("random jobs");
