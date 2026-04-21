use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use resource_manager::{
    self as rm,
    program::ResourceManager,
};
use crate::state::PlayerRecord;
use crate::errors::ForageError;

#[derive(Accounts)]
pub struct ForageMaterials<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"player_record", player.key().as_ref()],
        bump = record.bump,
        constraint = record.owner == player.key() @ ForageError::InvalidPlayer,
    )]
    pub record: Account<'info, PlayerRecord>,
    
    /// CHECK: Authorized CPI signer
    #[account(seeds = [b"cpi_auth"], bump)]
    pub cpi_auth: UncheckedAccount<'info>,
    
    #[account(
        seeds = [b"world_state"],
        bump = world_state.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub world_state: Account<'info, rm::WorldState>,
    
    /// CHECK: Material authority PDA from resource_manager
    #[account(
        seeds = [b"material_auth"],
        bump = world_state.material_auth_bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub material_auth: UncheckedAccount<'info>,
    
    pub resource_manager_program: Program<'info, ResourceManager>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}