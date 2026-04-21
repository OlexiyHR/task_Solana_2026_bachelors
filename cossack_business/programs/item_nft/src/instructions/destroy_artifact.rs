use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{
    state::{ArtifactConfig, ArtifactRecord},
    errors::ArtifactError,
    constants::MPL_TOKEN_METADATA_ID,
};

#[derive(Accounts)]
pub struct DestroyArtifact<'info> {
    #[account(
        seeds = [b"cpi_auth"],
        bump,
        seeds::program = config.trade_program,
    )]
    pub cpi_auth: Signer<'info>,
    
    #[account(seeds = [b"artifact_config"], bump = config.bump)]
    pub config: Account<'info, ArtifactConfig>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        mut,
        close = player,
        seeds = [b"artifact_meta", nft_mint.key().as_ref()],
        bump = artifact_metadata.bump,
        constraint = artifact_metadata.owner == player.key() @ ArtifactError::OwnershipMismatch,
    )]
    pub artifact_metadata: Account<'info, ArtifactRecord>,
    
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = player,
    )]
    pub player_nft_ata: Account<'info, TokenAccount>,
    
    /// CHECK: Metaplex
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: Metaplex
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: MPL Program
    #[account(address = MPL_TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,
    /// CHECK: Sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}