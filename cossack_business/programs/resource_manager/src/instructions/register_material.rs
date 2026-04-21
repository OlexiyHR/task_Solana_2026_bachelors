use anchor_lang::prelude::*;
use crate::state::WorldState;
use crate::errors::WorldError;

#[derive(Accounts)]
#[instruction(material_idx: u8)]
pub struct RegisterMaterial<'info> {
    #[account(
        mut,
        constraint = authority.key() == world_state.authority @ WorldError::AccessDenied,
    )]
    pub authority: Signer<'info>,
    
    #[account(mut, seeds = [b"world_state"], bump = world_state.bump)]
    pub world_state: Account<'info, WorldState>,
    
    #[account(mut)]
    pub mint: Signer<'info>,
    
    /// CHECK: Validated via seeds constraint
    #[account(seeds = [b"material_auth"], bump = world_state.material_auth_bump)]
    pub material_auth: UncheckedAccount<'info>,
    
    /// CHECK: Token-2022 program address verification
    #[account(address = spl_token_2022::ID)]
    pub token_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}