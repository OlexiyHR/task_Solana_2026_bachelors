use anchor_lang::prelude::*;
use crate::state::WorldState;

#[derive(Accounts)]
pub struct InitWorld<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + WorldState::INIT_SPACE,
        seeds = [b"world_state"],
        bump,
    )]
    pub world_state: Account<'info, WorldState>,
    
    /// CHECK: PDA for managing all Token-2022 material mints
    #[account(seeds = [b"material_auth"], bump)]
    pub material_auth: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}