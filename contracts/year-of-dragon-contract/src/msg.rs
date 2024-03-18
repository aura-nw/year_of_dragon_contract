use cosmwasm_schema::{cw_serde, QueryResponses};
use nois::NoisCallback;

use crate::state::Config;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    // must be hex string and has length 64
    pub random_seed: String,
    // bench32 string address
    pub nois_proxy: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // Forging gem
    ForgeGem { request_forge_hash: String },
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
    // Query Random seed from request forge hash
    #[returns(String)]
    RandomSeedFromRequestForgeHash { request_forge_hash: String },
}
