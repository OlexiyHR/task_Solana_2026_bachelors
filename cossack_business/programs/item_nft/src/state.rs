use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ArtifactConfig {
    pub admin: Pubkey,
    pub forge_program: Pubkey,
    pub trade_program: Pubkey,
    pub bump: u8,
    pub artifact_auth_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ArtifactRecord {
    pub artifact_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}