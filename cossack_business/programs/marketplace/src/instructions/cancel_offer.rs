use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::TradeOffer;
use crate::errors::TradeError;

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"trade_offer", offer.artifact_mint.as_ref()],
        bump = offer.bump,
        constraint = offer.creator == seller.key() @ TradeError::NotArtifactOwner,
        close = seller,
    )]
    pub offer: Account<'info, TradeOffer>,

    pub artifact_mint: Account<'info, Mint>,

    /// CHECK: Escrow Authority
    #[account(
        seeds = [b"escrow_auth", artifact_mint.key().as_ref()],
        bump = offer.escrow_bump,
    )]
    pub escrow_auth: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = artifact_mint,
        token::authority = escrow_auth,
    )]
    pub escrow_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = artifact_mint,
        token::authority = seller,
    )]
    pub seller_artifact_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}