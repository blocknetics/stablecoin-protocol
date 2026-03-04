use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Open a new collateral vault, deposit SOL, and mint stablecoins.
pub fn handler(
    ctx: Context<OpenVault>,
    collateral_amount: u64,
    mint_amount: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.is_shutdown, StableError::ProtocolShutdown);

    // Transfer SOL from user to collateral vault PDA
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
            },
        ),
        collateral_amount,
    )?;

    // Verify collateral ratio meets minimum
    let vault = &mut ctx.accounts.vault;
    vault.owner = ctx.accounts.owner.key();
    vault.collateral_amount = collateral_amount;
    vault.debt_amount = mint_amount;
    vault.last_interest_accrual = Clock::get()?.unix_timestamp;
    vault.bump = ctx.bumps.vault;

    let ratio = vault.collateral_ratio_bps(config.oracle_price);
    require!(
        ratio >= config.collateral_ratio_bps,
        StableError::BelowCollateralRatio
    );

    // Mint stablecoins to user
    let config_seeds: &[&[u8]] = &[b"config", &[config.bump]];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                to: ctx.accounts.user_stablecoin_account.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            &[config_seeds],
        ),
        mint_amount,
    )?;

    // Update protocol totals
    config.total_debt = config.total_debt.checked_add(mint_amount).ok_or(StableError::MathOverflow)?;
    config.total_collateral = config.total_collateral.checked_add(collateral_amount).ok_or(StableError::MathOverflow)?;

    emit!(VaultOpened {
        owner: ctx.accounts.owner.key(),
        collateral_amount,
        debt_amount: mint_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct OpenVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    /// New vault PDA seeded by owner
    #[account(
        init,
        payer = owner,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", owner.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"stablecoin-mint"],
        bump = config.mint_bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// The user's stablecoin token account
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// PDA collateral vault
    /// CHECK: Seeds-validated SOL holder
    #[account(
        mut,
        seeds = [b"collateral-vault"],
        bump = config.vault_bump,
    )]
    pub collateral_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
