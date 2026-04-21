use anchor_lang::prelude::*;

/// Persistent on-chain record for a registered player.
#[account]
#[derive(InitSpace)]
pub struct PlayerRecord {
    pub owner: Pubkey,
    pub last_forage_time: i64,
    pub bump: u8,
}