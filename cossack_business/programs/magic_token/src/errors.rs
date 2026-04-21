use anchor_lang::prelude::*;

#[error_code]
pub enum CurrencyError {
    #[msg("The requested mint amount must be greater than zero")]
    InvalidMintAmount,
    #[msg("The provided mint account does not match the registered currency mint")]
    UnrecognizedMint,
    #[msg("Failed to calculate the required space for the metadata pointer")]
    SpaceAllocationFailed,
}