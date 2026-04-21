use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
    token_interface::{
        Mint as MintInterface,
        TokenAccount as TokenAccountInterface,
        TokenInterface,
    },
};
use item_nft::program::ItemNft;
use crate::state::TradeOffer;
use crate::errors::TradeError;

#[derive(Accounts)]
pub struct ExecuteTrade<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(seeds = [b"cpi_auth"], bump)]
    pub cpi_auth: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"trade_offer", offer.artifact_mint.as_ref()],
        bump = offer.bump,
        close = seller,
    )]
    pub offer: Box<Account<'info, TradeOffer>>,

    /// CHECK: Validated against offer
    #[account(
        mut,
        constraint = seller.key() == offer.creator @ TradeError::CreatorMismatch,
    )]
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"artifact_meta", offer.artifact_mint.as_ref()],
        bump,
        seeds::program = item_nft::ID,
    )]
    pub artifact_record: Box<Account<'info, item_nft::ArtifactRecord>>,

    #[account(
        seeds = [b"artifact_config"],
        bump = artifact_config.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub artifact_config: Box<Account<'info, item_nft::ArtifactConfig>>,

    #[account(
        constraint = artifact_mint.key() == offer.artifact_mint @ TradeError::CurrencyMintMismatch,
    )]
    pub artifact_mint: Box<Account<'info, Mint>>,

    /// CHECK: Escrow PDA
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
    pub escrow_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = artifact_mint,
        token::authority = buyer,
    )]
    pub buyer_artifact_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"currency_state"],
        bump = currency_state.bump,
        seeds::program = magic_token::ID,
    )]
    pub currency_state: Box<Account<'info, magic_token::CurrencyState>>,

    #[account(
        constraint = currency_mint.key() == currency_state.token_mint @ TradeError::CurrencyMintMismatch,
    )]
    pub currency_mint: Box<InterfaceAccount<'info, MintInterface>>,

    #[account(mut)]
    pub buyer_currency_ata: Box<InterfaceAccount<'info, TokenAccountInterface>>,

    #[account(mut)]
    pub seller_currency_ata: Box<InterfaceAccount<'info, TokenAccountInterface>>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}