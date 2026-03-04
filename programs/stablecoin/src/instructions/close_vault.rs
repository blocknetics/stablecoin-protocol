use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Close a vault: burn all outstanding debt and return collateral.
pub fn handler(ctx: Context<CloseVault>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let vault = &mut ctx.accounts.vault;

    // Accrue interest before closing
    vault.accrue_interest(config.stability_fee_bps, Clock::get()?.unix_timestamp);

    let debt = vault.debt_amount;
    let collateral = vault.collateral_amount;

    require!(debt > 0 || collateral > 0, StableError::InsufficientDebt);

    // Burn stablecoins from user to repay debt
    if debt > 0 {
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.stablecoin_mint.to_account_info(),
                    from: ctx.accounts.user_stablecoin_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            debt,
        )?;
    }

    // Return collateral SOL from vault PDA to user
    if collateral > 0 {
        let vault_bump = config.vault_bump;
        let signer_seeds: &[&[u8]] = &[b"collateral-vault", &[vault_bump]];
        **ctx.accounts.collateral_vault.to_account_info().try_borrow_mut_lamports()? -= collateral;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += collateral;
    }

    // Update protocol totals
    config.total_debt = config.total_debt.saturating_sub(debt);
    config.total_collateral = config.total_collateral.saturating_sub(collateral);

    // Zero out the vault
    vault.collateral_amount = 0;
    vault.debt_amount = 0;

    emit!(VaultClosed {
        owner: ctx.accounts.owner.key(),
        collateral_returned: collateral,
        debt_repaid: debt,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner,
        close = owner,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"stablecoin-mint"],
        bump = config.mint_bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// CHECK: Seeds-validated SOL holder
    #[account(
        mut,
        seeds = [b"collateral-vault"],
        bump = config.vault_bump,
    )]
    pub collateral_vault: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
