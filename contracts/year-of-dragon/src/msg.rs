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
    ForgeGem {
        request_forge_hash: String,
    },
    // Select Jackpot Gems
    SelectJackpotGems {
        campaign_id: String,
        max_star: String,
        max_number: String,
    },
    // Nois callback
    NoisReceive {
        callback: NoisCallback,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    // Query Random seed from request forge hash
    #[returns(RandomResponse)]
    RandomSeedFromRequestForgeHash { request_forge_hash: String },
    // Query Jackpot Gems from campaign id
    #[returns(String)]
    GetJackpotGems { campaign_id: String },
}
