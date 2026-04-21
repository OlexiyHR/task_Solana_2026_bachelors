use anchor_lang::prelude::*;
use crate::state::ArtifactConfig;

#[derive(Accounts)]
pub struct InitArtifactSystem<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + ArtifactConfig::INIT_SPACE,
        seeds = [b"artifact_config"],
        bump,
    )]
    pub config: Account<'info, ArtifactConfig>,
    
    /// CHECK: PDA for managing all Artifact Mints
    #[account(seeds = [b"artifact_auth"], bump)]
    pub nft_authority: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}