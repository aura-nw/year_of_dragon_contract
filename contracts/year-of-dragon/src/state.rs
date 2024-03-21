use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
    pub contract_operator: Addr,
}

#[cw_serde]
pub struct RandomJob {
    pub request_forge_hash: String,
}

#[cw_serde]
pub struct RandomResponse {
    pub request_forge_hash: String,
    pub random_seed: String,
    pub drand_round: String,
}

#[cw_serde]
pub struct RandomJobs {
    pub randomness: String,
    pub action: String,
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

pub const CONFIG: Item<Config> = Item::new("config");

// https://api3.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/info
pub const DRAND_GENESIS: Timestamp = Timestamp::from_seconds(1677685200);
pub const DRAND_ROUND_LENGTH: u64 = 3_000_000_000; // in nanoseconds
pub const RANDOM_SEED: Item<String> = Item::new("random seed");
pub const RANDOM_JOBS: Map<String, RandomJobs> = Map::new("random jobs");
pub const DRAND_ROUND_WITH_HASH: Map<String, String> = Map::new("drand round with hash");
pub const MAX_STAR_WITH_CAMPAIGN_ID: Map<String, u32> = Map::new("max star with campaign id");
pub const JACKPOT_GEMS_WITH_CAMPAIGN_ID: Map<String, String> =
    Map::new("jackpot gems with campaign id");
