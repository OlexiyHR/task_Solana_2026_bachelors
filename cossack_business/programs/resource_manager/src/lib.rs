use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    system_instruction,
};
use spl_token_2022::{extension::ExtensionType, instruction::initialize_mint2};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("8xTTGum6kEZtbn6f6vVu3go2YWLwc9r2mbvmRKY23aCj");

#[program]
pub mod resource_manager {
    use super::*;

    pub fn init_world(
        ctx: Context<InitWorld>,
        artifact_values: [u64; 4],
        drop_chances: [u8; 6],
        forage_cooldown: i64,
        forage_program: Pubkey,
        forge_program: Pubkey,
        trade_program: Pubkey,
    ) -> Result<()> {
        let sum: u16 = drop_chances.iter().map(|&w| w as u16).sum();
        require!(sum == 100, WorldError::InvalidDropChances);
        require!(forage_cooldown > 0, WorldError::InvalidTimerConfig);

        let state = &mut ctx.accounts.world_state;
        state.authority = ctx.accounts.authority.key();
        state.artifact_values = artifact_values;
        state.drop_chances = drop_chances;
        state.forage_cooldown = forage_cooldown;
        state.forage_program = forage_program;
        state.forge_program = forge_program;
        state.trade_program = trade_program;
        state.registered_materials = 0;
        state.bump = ctx.bumps.world_state;
        state.material_auth_bump = ctx.bumps.material_auth;

        Ok(())
    }

    pub fn register_material(
        ctx: Context<RegisterMaterial>,
        material_idx: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(material_idx < MATERIAL_LIMIT as u8, WorldError::InvalidMaterialIndex);
        require!(material_idx == ctx.accounts.world_state.registered_materials, WorldError::NonSequentialRegistration);

        let mint_pk = ctx.accounts.mint.key();
        let auth_pk = ctx.accounts.material_auth.key();

        let space_needed = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MetadataPointer,
        ]).map_err(|_| WorldError::AllocationError)?;

        let rent = Rent::get()?;
        let base_lamports = rent.minimum_balance(space_needed);

        invoke(
            &system_instruction::create_account(&ctx.accounts.authority.key(), &mint_pk, base_lamports, space_needed as u64, &spl_token_2022::ID),
            &[ctx.accounts.authority.to_account_info(), ctx.accounts.mint.to_account_info()],
        )?;

        invoke(
            &spl_token_2022::extension::metadata_pointer::instruction::initialize(&spl_token_2022::ID, &mint_pk, Some(auth_pk), Some(mint_pk))?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        invoke(
            &initialize_mint2(&spl_token_2022::ID, &mint_pk, &auth_pk, None, 0)?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        let meta = TokenMetadata {
            name: name.clone(), symbol: symbol.clone(), uri: uri.clone(), mint: mint_pk,
            update_authority: Some(auth_pk).try_into().unwrap(), additional_metadata: vec![],
        };

        let packed_len = meta.get_packed_len().unwrap_or(256);
        let total_space = space_needed + 12 + packed_len;
        let extra_lamports = rent.minimum_balance(total_space).saturating_sub(base_lamports);

        if extra_lamports > 0 {
            invoke(
                &system_instruction::transfer(&ctx.accounts.authority.key(), &mint_pk, extra_lamports),
                &[ctx.accounts.authority.to_account_info(), ctx.accounts.mint.to_account_info()],
            )?;
        }

        let bump = ctx.accounts.world_state.material_auth_bump;
        let signer_seeds: &[&[u8]] = &[b"material_auth", &[bump]];

        invoke_signed(
            &spl_token_metadata_interface::instruction::initialize(
                &spl_token_2022::ID, &mint_pk, &auth_pk, &mint_pk, &auth_pk, name, symbol, uri,
            ),
            &[ctx.accounts.mint.to_account_info(), ctx.accounts.material_auth.to_account_info()],
            &[signer_seeds],
        )?;

        let state = &mut ctx.accounts.world_state;
        state.material_mints[material_idx as usize] = mint_pk;
        state.registered_materials += 1;

        Ok(())
    }

    pub fn issue_material(ctx: Context<IssueMaterial>, material_idx: u8, amount: u64) -> Result<()> {
        require!(material_idx < MATERIAL_LIMIT as u8, WorldError::InvalidMaterialIndex);
        require!(amount > 0, WorldError::ZeroAmountRequested);

        let bump = ctx.accounts.world_state.material_auth_bump;
        let signer_seeds: &[&[u8]] = &[b"material_auth", &[bump]];

        anchor_spl::token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::MintTo {
                    mint: ctx.accounts.material_mint.to_account_info(),
                    to: ctx.accounts.receiver_ata.to_account_info(),
                    authority: ctx.accounts.material_auth.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )
    }

    pub fn consume_material(ctx: Context<ConsumeMaterial>, material_idx: u8, amount: u64) -> Result<()> {
        require!(material_idx < MATERIAL_LIMIT as u8, WorldError::InvalidMaterialIndex);
        require!(amount > 0, WorldError::ZeroAmountRequested);

        anchor_spl::token_2022::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::Burn {
                    mint: ctx.accounts.material_mint.to_account_info(),
                    from: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            amount,
        )
    }

    pub fn adjust_drop_rates(ctx: Context<SystemAdmin>, new_rates: [u8; 6]) -> Result<()> {
        let sum: u16 = new_rates.iter().map(|&w| w as u16).sum();
        require!(sum == 100, WorldError::InvalidDropChances);
        ctx.accounts.world_state.drop_chances = new_rates;
        Ok(())
    }

    pub fn set_timer(ctx: Context<SystemAdmin>, new_timer: i64) -> Result<()> {
        require!(new_timer > 0, WorldError::InvalidTimerConfig);
        ctx.accounts.world_state.forage_cooldown = new_timer;
        Ok(())
    }
}
