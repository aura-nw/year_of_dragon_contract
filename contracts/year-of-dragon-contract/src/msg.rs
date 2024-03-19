use cosmwasm_schema::{cw_serde, QueryResponses};
use nois::NoisCallback;

use crate::state::{Config, RandomResponse};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    // bench32 string address
    pub nois_proxy: String,
    // operator address
    pub operator: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // Forging gem
    ForgeGem { request_forge_hash: String },
    // Get Jackpot Gems
    GetJackpotGems { request_get_jackpot_hash: String },
    // Nois callback
    NoisReceive { callback: NoisCallback },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    // Random seed
    #[returns(String)]
    RandomSeed {},
    // Query Random seed from request hash
    #[returns(RandomResponse)]
    RandomSeedFromRequestHash { request_hash: String },
}
