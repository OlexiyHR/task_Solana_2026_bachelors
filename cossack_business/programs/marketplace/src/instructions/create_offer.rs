use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::state::TradeOffer;
use crate::errors::TradeError;

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        constraint = artifact_record.owner == seller.key() @ TradeError::NotArtifactOwner,
    )]
    pub artifact_record: Account<'info, item_nft::ArtifactRecord>,

    pub artifact_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = artifact_mint,
        token::authority = seller,
    )]
    pub seller_artifact_ata: Account<'info, TokenAccount>,

    /// CHECK: Escrow PDA authority
    #[account(seeds = [b"escrow_auth", artifact_mint.key().as_ref()], bump)]
    pub escrow_auth: UncheckedAccount<'info>,

    #[account(
        init,
        payer = seller,
        associated_token::mint = artifact_mint,
        associated_token::authority = escrow_auth,
    )]
    pub escrow_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = seller,
        space = 8 + TradeOffer::INIT_SPACE,
        seeds = [b"trade_offer", artifact_mint.key().as_ref()],
        bump,
    )]
    pub offer: Account<'info, TradeOffer>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
