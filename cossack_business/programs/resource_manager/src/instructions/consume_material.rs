use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::WorldState;
use crate::errors::WorldError;

#[derive(Accounts)]
#[instruction(material_idx: u8)]
pub struct ConsumeMaterial<'info> {
    #[account(
        seeds = [b"cpi_auth"],
        bump,
        seeds::program = world_state.forge_program,
    )]
    pub cpi_auth: Signer<'info>,
    
    pub player: Signer<'info>,
    
    #[account(seeds = [b"world_state"], bump = world_state.bump)]
    pub world_state: Account<'info, WorldState>,
    
    #[account(
        mut,
        constraint = material_mint.key() == world_state.material_mints[material_idx as usize]
            @ WorldError::UnrecognizedMint,
    )]
    pub material_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        token::mint = material_mint,
        token::authority = player,
    )]
    pub player_ata: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
}