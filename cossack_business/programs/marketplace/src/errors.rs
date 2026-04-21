use anchor_lang::prelude::*;

#[error_code]
pub enum TradeError {
    #[msg("Offer price must be greater than zero")]
    ZeroPriceNotAllowed,
    #[msg("Signer is not the recorded owner of this artifact")]
    NotArtifactOwner,
    #[msg("The signer does not match the creator of this trade offer")]
    CreatorMismatch,
    #[msg("Artifact type index is invalid")]
    InvalidArtifactType,
    #[msg("Mint address does not match the configured currency mint")]
    CurrencyMintMismatch,
    #[msg("Incorrect number of remaining accounts provided for CPI")]
    MissingCpiAccounts,
}