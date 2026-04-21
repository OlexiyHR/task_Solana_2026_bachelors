use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    system_instruction,
};
use spl_token_2022::{extension::ExtensionType, instruction::initialize_mint2};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("7dXKNT6HXVvswbDCtzcLsAaPBa4jsPXnfhF1oP7y3Rgs");

#[program]
pub mod magic_token {
    use super::*;

    pub fn init_currency(
        ctx: Context<InitCurrency>,
        name: String,
        symbol: String,
        uri: String,
        trade_program: Pubkey,
    ) -> Result<()> {
        let mint_pk = ctx.accounts.mint.key();
        let auth_pk = ctx.accounts.mint_authority.key();

        let space_needed =
            ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
                ExtensionType::MetadataPointer,
            ])
            .map_err(|_| CurrencyError::SpaceAllocationFailed)?;

        let rent = Rent::get()?;
        let base_lamports = rent.minimum_balance(space_needed);

        invoke(
            &system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &mint_pk,
                base_lamports,
                space_needed as u64,
                &spl_token_2022::ID,
            ),
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ],
        )?;

        invoke(
            &spl_token_2022::extension::metadata_pointer::instruction::initialize(
                &spl_token_2022::ID,
                &mint_pk,
                Some(auth_pk),
                Some(mint_pk),
            )?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        invoke(
            &initialize_mint2(
                &spl_token_2022::ID,
                &mint_pk,
                &auth_pk,
                None,
                0,
            )?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        let meta = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            mint: mint_pk,
            update_authority: Some(auth_pk).try_into().unwrap(),
            additional_metadata: vec![],
        };

        let packed_len = meta.get_packed_len().unwrap_or(256);
        let total_space = space_needed + 12 + packed_len;
        let extra_lamports = rent
            .minimum_balance(total_space)
            .saturating_sub(base_lamports);

        if extra_lamports > 0 {
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.admin.key(),
                    &mint_pk,
                    extra_lamports,
                ),
                &[
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.mint.to_account_info(),
                ],
            )?;
        }

        let bump = ctx.bumps.mint_authority;
        let signer_seeds: &[&[u8]] = &[b"currency_auth", &[bump]];

        invoke_signed(
            &spl_token_metadata_interface::instruction::initialize(
                &spl_token_2022::ID,
                &mint_pk,
                &auth_pk,
                &mint_pk,
                &auth_pk,
                name,
                symbol,
                uri,
            ),
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        let state = &mut ctx.accounts.config;
        state.admin = ctx.accounts.admin.key();
        state.token_mint = mint_pk;
        state.trade_program = trade_program;
        state.bump = ctx.bumps.config;
        state.auth_bump = ctx.bumps.mint_authority;

        Ok(())
    }

    pub fn issue_currency(
        ctx: Context<IssueCurrency>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, CurrencyError::InvalidMintAmount);

        let bump = ctx.accounts.config.auth_bump;
        let signer_seeds: &[&[u8]] = &[b"currency_auth", &[bump]];

        anchor_spl::token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::MintTo {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.receiver_ata.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )
    }
}
