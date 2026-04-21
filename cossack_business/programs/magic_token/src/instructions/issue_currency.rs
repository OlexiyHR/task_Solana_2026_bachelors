use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::CurrencyState;
use crate::errors::CurrencyError;

#[derive(Accounts)]
pub struct IssueCurrency<'info> {
    /// CPI signer from the trade program
    #[account(
        seeds = [b"cpi_auth"],
        bump,
        seeds::program = config.trade_program,
    )]
    pub cpi_auth: Signer<'info>,

    #[account(seeds = [b"currency_state"], bump = config.bump)]
    pub config: Account<'info, CurrencyState>,

    #[account(
        mut,
        constraint = token_mint.key() == config.token_mint @ CurrencyError::UnrecognizedMint,
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Must match the derived authority PDA
    #[account(seeds = [b"currency_auth"], bump = config.auth_bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub receiver_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}