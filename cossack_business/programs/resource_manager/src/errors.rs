use anchor_lang::prelude::*;

#[error_code]
pub enum WorldError {
    #[msg("Material index is out of valid bounds (0-5)")]
    InvalidMaterialIndex,
    #[msg("Materials must be registered strictly in sequential order")]
    NonSequentialRegistration,
    #[msg("Operation requires a non-zero token amount")]
    ZeroAmountRequested,
    #[msg("The provided mint address is not recognized in the world state")]
    UnrecognizedMint,
    #[msg("Signer lacks administrative privileges")]
    AccessDenied,
    #[msg("The sum of drop chances must be exactly 100%")]
    InvalidDropChances,
    #[msg("Cooldown timer must be greater than 0 seconds")]
    InvalidTimerConfig,
    #[msg("Failed to calculate required space for Token-2022 metadata")]
    AllocationError,
}