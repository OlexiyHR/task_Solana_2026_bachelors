use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::state::{ArtifactConfig, ArtifactRecord};
use crate::constants::MPL_TOKEN_METADATA_ID;

#[derive(Accounts)]
#[instruction(artifact_id: u8)]
pub struct IssueArtifact<'info> {
    #[account(
        seeds = [b"cpi_auth"],
        bump,
        seeds::program = config.forge_program,
    )]
    pub cpi_auth: Signer<'info>,
    
    #[account(seeds = [b"artifact_config"], bump = config.bump)]
    pub config: Account<'info, ArtifactConfig>,
    
    /// CHECK: Master authority for artifact NFTs
    #[account(seeds = [b"artifact_auth"], bump = config.artifact_auth_bump)]
    pub nft_authority: UncheckedAccount<'info>,
    
    /// CHECK: Wallet of the receiving player
    pub player: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = nft_authority,
        mint::freeze_authority = nft_authority,
    )]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
    )]
    pub player_nft_ata: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = payer,
        space = 8 + ArtifactRecord::INIT_SPACE,
        seeds = [b"artifact_meta", nft_mint.key().as_ref()],
        bump,
    )]
    pub artifact_metadata: Account<'info, ArtifactRecord>,
    
    /// CHECK: Metaplex metadata
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: Metaplex edition
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: MPL Program
    #[account(address = MPL_TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}