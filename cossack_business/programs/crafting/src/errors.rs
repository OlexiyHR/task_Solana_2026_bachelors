use anchor_lang::prelude::*;

#[error_code]
pub enum ForgeError {
    #[msg("Requested artifact does not have a valid blueprint")]
    UnknownArtifactBlueprint,
    #[msg("Provided extra accounts do not match required materials")]
    MismatchedAccountLength,
    #[msg("A required material for this blueprint was omitted from the transaction")]
    MaterialMissing,
    #[msg("The same material index was provided multiple times")]
    DuplicateMaterialIndex,
}