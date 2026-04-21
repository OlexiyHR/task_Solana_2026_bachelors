use anchor_lang::prelude::*;
use crate::state::{ArtifactConfig, ArtifactRecord};

#[derive(Accounts)]
pub struct UpdateArtifactOwner<'info> {
    #[account(
        seeds = [b"cpi_auth"],
        bump,
        seeds::program = config.trade_program,
    )]
    pub cpi_auth: Signer<'info>,
    
    #[account(seeds = [b"artifact_config"], bump = config.bump)]
    pub config: Account<'info, ArtifactConfig>,
    
    #[account(
        mut,
        seeds = [b"artifact_meta", artifact_metadata.mint.as_ref()],
        bump = artifact_metadata.bump,
    )]
    pub artifact_metadata: Account<'info, ArtifactRecord>,
}