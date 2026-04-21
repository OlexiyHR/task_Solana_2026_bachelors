use anchor_lang::prelude::*;

/// Represents an active trade offer holding an Artifact in escrow.
#[account]
#[derive(InitSpace)]
pub struct TradeOffer {
    pub creator: Pubkey,
    pub artifact_mint: Pubkey,
    pub artifact_type: u8,
    pub requested_price: u64,
    pub bump: u8,
    pub escrow_bump: u8,
}