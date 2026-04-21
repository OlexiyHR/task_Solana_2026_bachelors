use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct WorldState {
    pub authority: Pubkey,
    pub material_mints: [Pubkey; 6],
    pub forage_program: Pubkey,
    pub forge_program: Pubkey,
    pub trade_program: Pubkey,
    pub artifact_values: [u64; 4],
    pub drop_chances: [u8; 6],
    pub forage_cooldown: i64,
    pub registered_materials: u8,
    pub bump: u8,
    pub material_auth_bump: u8,
}