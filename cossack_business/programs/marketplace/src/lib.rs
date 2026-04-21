use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("5imzGijn2kip1VWCQtuLJXNsUc6vVV8UqpZSFSd1v6aD");

#[program]
pub mod marketplace {
    use super::*;

    pub fn liquidate_artifact<'info>(
        ctx: Context<'_, '_, 'info, 'info, LiquidateArtifact<'info>>,
    ) -> Result<()> {
        let a_type = ctx.accounts.artifact_record.artifact_type;
        require!(
            (a_type as usize) < ctx.accounts.world_state.artifact_values.len(),
            TradeError::InvalidArtifactType
        );
        
        let payout = ctx.accounts.world_state.artifact_values[a_type as usize];
        
        let auth_bump = ctx.bumps.cpi_auth;
        let signer_seeds: &[&[u8]] = &[b"cpi_auth", &[auth_bump]];
        
        let extras = ctx.remaining_accounts;
        require!(extras.len() >= 4, TradeError::MissingCpiAccounts);
        
        let metadata_account = extras[0].to_account_info();
        let master_edition = extras[1].to_account_info();
        let metadata_program = extras[2].to_account_info();
        let sysvar_instructions = extras[3].to_account_info();

        // Burn the Artifact via CPI
        item_nft::cpi::destroy_artifact(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::DestroyArtifact {
                    cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
                    config: ctx.accounts.artifact_config.to_account_info(),
                    player: ctx.accounts.player.to_account_info(),
                    artifact_metadata: ctx.accounts.artifact_record.to_account_info(),
                    nft_mint: ctx.accounts.artifact_mint.to_account_info(),
                    player_nft_ata: ctx.accounts.player_artifact_ata.to_account_info(),
                    metadata_account,
                    master_edition,
                    metadata_program,
                    sysvar_instructions,
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                &[signer_seeds],
            ),
        )?;

        // Mint Currency to Player via CPI
        magic_token::cpi::issue_currency(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                magic_token::cpi::accounts::IssueCurrency {
                    cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
                    config: ctx.accounts.currency_state.to_account_info(),
                    token_mint: ctx.accounts.currency_mint.to_account_info(),
                    mint_authority: ctx.accounts.currency_auth.to_account_info(),
                    receiver_ata: ctx.accounts.player_currency_ata.to_account_info(),
                    token_program: ctx.accounts.token_2022_program.to_account_info(),
                },
                &[signer_seeds],
            ),
            payout,
        )?;

        Ok(())
    }

    pub fn create_offer(ctx: Context<CreateOffer>, price: u64) -> Result<()> {
        require!(price > 0, TradeError::ZeroPriceNotAllowed);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.seller_artifact_ata.to_account_info(),
                    to: ctx.accounts.escrow_ata.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        let offer = &mut ctx.accounts.offer;
        offer.creator = ctx.accounts.seller.key();
        offer.artifact_mint = ctx.accounts.artifact_mint.key();
        offer.artifact_type = ctx.accounts.artifact_record.artifact_type;
        offer.requested_price = price;
        offer.bump = ctx.bumps.offer;
        offer.escrow_bump = ctx.bumps.escrow_auth;

        Ok(())
    }

    pub fn execute_trade(ctx: Context<ExecuteTrade>) -> Result<()> {
        let price = ctx.accounts.offer.requested_price;
        let mint_key = ctx.accounts.offer.artifact_mint;
        let e_bump = ctx.accounts.offer.escrow_bump;

        // Pay the seller
        anchor_spl::token_2022::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_2022_program.to_account_info(),
                anchor_spl::token_2022::TransferChecked {
                    from: ctx.accounts.buyer_currency_ata.to_account_info(),
                    mint: ctx.accounts.currency_mint.to_account_info(),
                    to: ctx.accounts.seller_currency_ata.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            price,
            0,
        )?;

        // Transfer Artifact to buyer
        let escrow_seeds: &[&[u8]] = &[b"escrow_auth", mint_key.as_ref(), &[e_bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_ata.to_account_info(),
                    to: ctx.accounts.buyer_artifact_ata.to_account_info(),
                    authority: ctx.accounts.escrow_auth.to_account_info(),
                },
                &[escrow_seeds],
            ),
            1,
        )?;

        // Update Artifact Owner Record
        let auth_bump = ctx.bumps.cpi_auth;
        let signer_seeds: &[&[u8]] = &[b"cpi_auth", &[auth_bump]];
        
        item_nft::cpi::update_artifact_owner(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::UpdateArtifactOwner {
                    cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
                    config: ctx.accounts.artifact_config.to_account_info(),
                    artifact_metadata: ctx.accounts.artifact_record.to_account_info(),
                },
                &[signer_seeds],
            ),
            ctx.accounts.buyer.key(),
        )?;

        Ok(())
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        let mint_key = ctx.accounts.offer.artifact_mint;
        let e_bump = ctx.accounts.offer.escrow_bump;
        let escrow_seeds: &[&[u8]] = &[b"escrow_auth", mint_key.as_ref(), &[e_bump]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_ata.to_account_info(),
                    to: ctx.accounts.seller_artifact_ata.to_account_info(),
                    authority: ctx.accounts.escrow_auth.to_account_info(),
                },
                &[escrow_seeds],
            ),
            1,
        )?;

        Ok(())
    }
}
