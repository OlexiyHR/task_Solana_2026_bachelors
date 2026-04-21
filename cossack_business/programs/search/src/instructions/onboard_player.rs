use anchor_lang::prelude::*;
use crate::state::PlayerRecord;

#[derive(Accounts)]
pub struct OnboardPlayer<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        init,
        payer = player,
        space = 8 + PlayerRecord::INIT_SPACE,
        seeds = [b"player_record", player.key().as_ref()],
        bump,
    )]
    pub record: Account<'info, PlayerRecord>,
    
    pub system_program: Program<'info, System>,
}