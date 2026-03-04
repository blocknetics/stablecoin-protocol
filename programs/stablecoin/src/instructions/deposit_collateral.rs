use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Deposit additional SOL collateral into an existing vault.
pub fn handler(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
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
        amount,
    )?;

    let vault = &mut ctx.accounts.vault;
    vault.collateral_amount = vault
        .collateral_amount
        .checked_add(amount)
        .ok_or(StableError::MathOverflow)?;

    config.total_collateral = config
        .total_collateral
        .checked_add(amount)
        .ok_or(StableError::MathOverflow)?;

    emit!(CollateralDeposited {
        owner: ctx.accounts.owner.key(),
        amount,
        new_collateral_total: vault.collateral_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
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
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: Seeds-validated SOL holder
    #[account(
        mut,
        seeds = [b"collateral-vault"],
        bump = config.vault_bump,
    )]
    pub collateral_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
