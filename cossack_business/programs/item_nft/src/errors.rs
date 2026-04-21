use anchor_lang::prelude::*;

#[error_code]
pub enum ArtifactError {
    #[msg("Artifact type ID is out of the valid range (0-3)")]
    UnknownArtifactType,
    #[msg("The transaction signer is not the registered owner of this artifact")]
    OwnershipMismatch,
}