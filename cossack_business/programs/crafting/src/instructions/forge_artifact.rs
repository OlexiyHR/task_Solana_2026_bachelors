use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::TokenInterface,
};
use item_nft::program::ItemNft;
use resource_manager::{
    self as rm,
    program::ResourceManager,
};

#[derive(Accounts)]
pub struct ForgeArtifact<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: Authorized CPI signer
    #[account(seeds = [b"cpi_auth"], bump)]
    pub cpi_auth: UncheckedAccount<'info>,

    #[account(
        seeds = [b"world_state"],
        bump = world_state.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub world_state: Account<'info, rm::WorldState>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}