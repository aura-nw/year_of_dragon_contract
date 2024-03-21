use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid address")]
    InvalidAddress {},

    #[error("Invalid randomness")]
    InvalidRandomness {},

    #[error("Invalid campaign id")]
    InvalidCampaignId {},

    #[error("Invalid forge hash")]
    InvalidForgeHash {},

    #[error("Invalid max star")]
    InvalidMaxStar {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
