use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::token_interface::{
    Mint as MintInterface,
    TokenAccount as TokenAccountInterface,
    TokenInterface,
};
use item_nft::program::ItemNft;
use magic_token::program::MagicToken;
use resource_manager::{self as rm};
use crate::errors::TradeError;

#[derive(Accounts)]
pub struct LiquidateArtifact<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: Authorized CPI signer
    #[account(seeds = [b"cpi_auth"], bump)]
    pub cpi_auth: UncheckedAccount<'info>,

    #[account(
        seeds = [b"world_state"],
        bump = world_state.bump,
        seeds::program = resource_manager::ID,
    )]
    pub world_state: Box<Account<'info, rm::WorldState>>,

    #[account(mut)]
    pub artifact_record: Box<Account<'info, item_nft::ArtifactRecord>>,

    #[account(mut)]
    pub artifact_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = artifact_mint,
        token::authority = player,
    )]
    pub player_artifact_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"artifact_config"],
        bump = artifact_config.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub artifact_config: Box<Account<'info, item_nft::ArtifactConfig>>,

    #[account(
        seeds = [b"currency_state"],
        bump = currency_state.bump,
        seeds::program = magic_token_program.key(),
    )]
    pub currency_state: Box<Account<'info, magic_token::CurrencyState>>,

    #[account(
        mut,
        constraint = currency_mint.key() == currency_state.token_mint @ TradeError::CurrencyMintMismatch,
    )]
    pub currency_mint: InterfaceAccount<'info, MintInterface>,

    /// CHECK: PDA mint authority for the currency token
    #[account(
        seeds = [b"currency_auth"],
        bump = currency_state.auth_bump,
        seeds::program = magic_token_program.key(),
    )]
    pub currency_auth: UncheckedAccount<'info>,

    #[account(mut)]
    pub player_currency_ata: InterfaceAccount<'info, TokenAccountInterface>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}