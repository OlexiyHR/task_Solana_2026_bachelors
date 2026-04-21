use anchor_lang::prelude::*;

/// Central configuration PDA for the game currency.
#[account]
#[derive(InitSpace)]
pub struct CurrencyState {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub trade_program: Pubkey,
    pub bump: u8,
    pub auth_bump: u8,
}