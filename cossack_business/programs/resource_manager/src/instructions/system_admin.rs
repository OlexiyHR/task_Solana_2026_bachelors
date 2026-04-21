use anchor_lang::prelude::*;
use crate::state::WorldState;
use crate::errors::WorldError;

#[derive(Accounts)]
pub struct SystemAdmin<'info> {
    #[account(constraint = authority.key() == world_state.authority @ WorldError::AccessDenied)]
    pub authority: Signer<'info>,
    
    #[account(mut, seeds = [b"world_state"], bump = world_state.bump)]
    pub world_state: Account<'info, WorldState>,
}