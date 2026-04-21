use anchor_lang::prelude::*;
use resource_manager::{self as rm, cpi::accounts::ConsumeMaterial};

pub mod constants;
pub mod errors;
pub mod instructions;

pub use constants::*;
pub use errors::*;
pub use instructions::*;

declare_id!("2yezAR5kHwvd8hL89RDnr9UnoHus84uzUuMsvac2ojze");

#[program]
pub mod crafting {
    use super::*;

    pub fn forge_artifact<'info>(
        ctx: Context<'_, '_, 'info, 'info, ForgeArtifact<'info>>,
        artifact_id: u8,
        material_indices: Vec<u8>,
    ) -> Result<()> {
        require!((artifact_id as usize) < BLUEPRINTS.len(), ForgeError::UnknownArtifactBlueprint);
        let blueprint = BLUEPRINTS[artifact_id as usize];

        let mut tracked = [false; MATERIAL_LIMIT];
        for &mid in material_indices.iter() {
            require!((mid as usize) < MATERIAL_LIMIT, ForgeError::UnknownArtifactBlueprint);
            require!(!tracked[mid as usize], ForgeError::DuplicateMaterialIndex);
            tracked[mid as usize] = true;
        }

        for (mid, &qty_needed) in blueprint.iter().enumerate() {
            if qty_needed > 0 {
                require!(
                    material_indices.contains(&(mid as u8)),
                    ForgeError::MaterialMissing
                );
            }
        }

        let auth_bump = ctx.bumps.cpi_auth;
        let auth_seeds: &[&[u8]] = &[b"cpi_auth", &[auth_bump]];
        let extras = ctx.remaining_accounts;
        let pair_count = material_indices.len();

        require!(
            extras.len() >= pair_count * 2 + 9,
            ForgeError::MismatchedAccountLength
        );

        // 1. Burn materials via resource_manager
        for (idx, &mid) in material_indices.iter().enumerate() {
            let qty = blueprint[mid as usize];
            if qty == 0 {
                continue;
            }

            let mint_acc = extras[idx * 2].to_account_info();
            let ata_acc = extras[idx * 2 + 1].to_account_info();

            rm::cpi::consume_material(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    ConsumeMaterial {
                        cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
                        player: ctx.accounts.player.to_account_info(),
                        world_state: ctx.accounts.world_state.to_account_info(),
                        material_mint: mint_acc,
                        player_ata: ata_acc,
                        token_program: ctx.accounts.token_2022_program.to_account_info(),
                    },
                    &[auth_seeds],
                ),
                mid,
                qty as u64,
            )?;
        }

        // 2. Mint the Artifact via item_nft
        let nft_start = pair_count * 2;
        let nft_mint = extras[nft_start].to_account_info();
        let player_nft_ata = extras[nft_start + 1].to_account_info();
        let metadata_account = extras[nft_start + 2].to_account_info();
        let master_edition = extras[nft_start + 3].to_account_info();
        let metadata_program = extras[nft_start + 4].to_account_info();
        let artifact_metadata = extras[nft_start + 5].to_account_info();
        let artifact_config = extras[nft_start + 6].to_account_info();
        let nft_authority = extras[nft_start + 7].to_account_info();
        let rent_sysvar = extras[nft_start + 8].to_account_info();

        item_nft::cpi::issue_artifact(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::IssueArtifact {
                    cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
                    config: artifact_config,
                    nft_authority,
                    player: ctx.accounts.player.to_account_info(),
                    payer: ctx.accounts.player.to_account_info(),
                    nft_mint,
                    player_nft_ata,
                    artifact_metadata,
                    metadata_account,
                    master_edition,
                    metadata_program,
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: rent_sysvar,
                },
                &[auth_seeds],
            ),
            artifact_id,
        )?;

        Ok(())
    }
}
