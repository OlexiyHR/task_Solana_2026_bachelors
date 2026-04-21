use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token::{self, MintTo};

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("HepLhfDjGV28DSL9bhHb9XNRZ2ShqYVtwJ9gFiasgt5s");

#[program]
pub mod item_nft {
    use super::*;

    pub fn init_artifact_system(
        ctx: Context<InitArtifactSystem>,
        forge_program: Pubkey,
        trade_program: Pubkey,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.admin = ctx.accounts.admin.key();
        cfg.forge_program = forge_program;
        cfg.trade_program = trade_program;
        cfg.bump = ctx.bumps.config;
        cfg.artifact_auth_bump = ctx.bumps.nft_authority;
        Ok(())
    }

    pub fn issue_artifact(ctx: Context<IssueArtifact>, artifact_id: u8) -> Result<()> {
        require!(artifact_id < ARTIFACT_TYPES_COUNT as u8, ArtifactError::UnknownArtifactType);

        let auth_bump = ctx.accounts.config.artifact_auth_bump;
        let signer_seeds: &[&[u8]] = &[b"artifact_auth", &[auth_bump]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.player_nft_ata.to_account_info(),
                    authority: ctx.accounts.nft_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            1,
        )?;

        let name = ARTIFACT_TITLES[artifact_id as usize].to_string();
        let symbol = ARTIFACT_TICKERS[artifact_id as usize].to_string();
        let uri = String::new();

        let meta_infos = vec![
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.nft_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let create_meta_ix = mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
            .metadata(ctx.accounts.metadata_account.key())
            .mint(ctx.accounts.nft_mint.key())
            .mint_authority(ctx.accounts.nft_authority.key())
            .payer(ctx.accounts.payer.key())
            .update_authority(ctx.accounts.nft_authority.key(), true)
            .system_program(ctx.accounts.system_program.key())
            .rent(Some(ctx.accounts.rent.key()))
            .data(mpl_token_metadata::types::DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            })
            .is_mutable(true)
            .instruction();

        invoke_signed(&create_meta_ix, &meta_infos, &[signer_seeds])?;

        let edition_infos = vec![
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let create_edition_ix = mpl_token_metadata::instructions::CreateMasterEditionV3Builder::new()
            .edition(ctx.accounts.master_edition.key())
            .mint(ctx.accounts.nft_mint.key())
            .update_authority(ctx.accounts.nft_authority.key())
            .mint_authority(ctx.accounts.nft_authority.key())
            .payer(ctx.accounts.payer.key())
            .metadata(ctx.accounts.metadata_account.key())
            .token_program(ctx.accounts.token_program.key())
            .system_program(ctx.accounts.system_program.key())
            .rent(Some(ctx.accounts.rent.key()))
            .max_supply(0)
            .instruction();

        invoke_signed(&create_edition_ix, &edition_infos, &[signer_seeds])?;

        let record = &mut ctx.accounts.artifact_metadata;
        record.artifact_type = artifact_id;
        record.owner = ctx.accounts.player.key();
        record.mint = ctx.accounts.nft_mint.key();
        record.bump = ctx.bumps.artifact_metadata;

        Ok(())
    }

    pub fn destroy_artifact(ctx: Context<DestroyArtifact>) -> Result<()> {
        let auth_bump = ctx.accounts.config.artifact_auth_bump;
        let signer_seeds: &[&[u8]] = &[b"artifact_auth", &[auth_bump]];

        let burn_infos = vec![
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.player_nft_ata.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
        ];

        let burn_ix = mpl_token_metadata::instructions::BurnV1Builder::new()
            .authority(ctx.accounts.player.key())
            .metadata(ctx.accounts.metadata_account.key())
            .edition(Some(ctx.accounts.master_edition.key()))
            .mint(ctx.accounts.nft_mint.key())
            .token(ctx.accounts.player_nft_ata.key())
            .spl_token_program(ctx.accounts.token_program.key())
            .system_program(ctx.accounts.system_program.key())
            .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
            .instruction();

        invoke_signed(&burn_ix, &burn_infos, &[signer_seeds])?;
        Ok(())
    }

    pub fn update_artifact_owner(
        ctx: Context<UpdateArtifactOwner>,
        new_owner: Pubkey,
    ) -> Result<()> {
        ctx.accounts.artifact_metadata.owner = new_owner;
        Ok(())
    }
}
