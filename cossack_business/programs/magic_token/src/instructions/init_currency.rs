use anchor_lang::prelude::*;
use crate::state::CurrencyState;

#[derive(Accounts)]
pub struct InitCurrency<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + CurrencyState::INIT_SPACE,
        seeds = [b"currency_state"],
        bump,
    )]
    pub config: Account<'info, CurrencyState>,

    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK: Derived PDA for mint authority
    #[account(seeds = [b"currency_auth"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Validated via ID
    #[account(address = spl_token_2022::ID)]
    pub token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}