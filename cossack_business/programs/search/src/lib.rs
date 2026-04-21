use anchor_lang::prelude::*;
use resource_manager::{self as rm, cpi::accounts::IssueMaterial};

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("6n4ojCjTvG9AGu7oxqQtXPTvw1Zwfn1gEGwgrmEPVrnm");

#[program]
pub mod search {
    use super::*;

    pub fn onboard_player(ctx: Context<OnboardPlayer>) -> Result<()> {
        let rec = &mut ctx.accounts.record;
        rec.owner = ctx.accounts.player.key();
        rec.last_forage_time = 0;
        rec.bump = ctx.bumps.record;
        Ok(())
    }

    pub fn forage_materials<'info>(
        ctx: Context<'_, '_, 'info, 'info, ForageMaterials<'info>>,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let rec = &mut ctx.accounts.record;

        let cooldown_sec = ctx.accounts.world_state.forage_cooldown;
        let time_passed = clock
            .unix_timestamp
            .checked_sub(rec.last_forage_time)
            .ok_or(ForageError::TimeCalculationFailed)?;
            
        require!(time_passed >= cooldown_sec, ForageError::CooldownActive);

        let random_seed = [
            clock.slot.to_le_bytes().as_ref(),
            clock.unix_timestamp.to_le_bytes().as_ref(),
            ctx.accounts.player.key().as_ref(),
        ]
        .concat();
        let hash_res = solana_program::hash::hashv(&[&random_seed]);
        let drop_chances = ctx.accounts.world_state.drop_chances;

        let auth_bump = ctx.bumps.cpi_auth;
        let signer_seeds: &[&[u8]] = &[b"cpi_auth", &[auth_bump]];
        
        let extras = ctx.remaining_accounts;
        require!(
            extras.len() == MATERIAL_LIMIT * 2,
            ForageError::IncorrectAccountCount
        );

        for drop_idx in 0..MATERIALS_PER_FORAGE {
            let mat_id = pick_weighted_material(hash_res.to_bytes()[drop_idx], &drop_chances);
            let mint_idx = mat_id as usize;
            let ata_idx = MATERIAL_LIMIT + mat_id as usize;

            rm::cpi::issue_material(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    IssueMaterial {
                        cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
                        world_state: ctx.accounts.world_state.to_account_info(),
                        material_mint: extras[mint_idx].to_account_info(),
                        material_auth: ctx.accounts.material_auth.to_account_info(),
                        receiver_ata: extras[ata_idx].to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                mat_id,
                1,
            )?;
        }

        rec.last_forage_time = clock.unix_timestamp;
        Ok(())
    }
}

fn pick_weighted_material(random_byte: u8, chances: &[u8; 6]) -> u8 {
    let roll = random_byte % 100;
    let mut sum = 0u8;
    for (i, &c) in chances.iter().enumerate() {
        sum = sum.saturating_add(c);
        if roll < sum {
            return i as u8;
        }
    }
    0
}
