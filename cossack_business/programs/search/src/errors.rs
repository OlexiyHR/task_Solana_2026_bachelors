use anchor_lang::prelude::*;

#[error_code]
pub enum ForageError {
    #[msg("Forage cooldown is still active. Please wait.")]
    CooldownActive,
    #[msg("Failed to calculate time difference safely")]
    TimeCalculationFailed,
    #[msg("The signer does not own this player record")]
    InvalidPlayer,
    #[msg("Incorrect number of remaining accounts provided for materials")]
    IncorrectAccountCount,
}