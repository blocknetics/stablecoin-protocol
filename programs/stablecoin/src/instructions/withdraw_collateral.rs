use anchor_lang::prelude::*;

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Withdraw excess collateral from a vault, maintaining the minimum ratio.
pub fn handler(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.is_shutdown, StableError::ProtocolShutdown);

    let vault = &mut ctx.accounts.vault;

    // Accrue interest before checking ratio
    vault.accrue_interest(config.stability_fee_bps, Clock::get()?.unix_timestamp);

    require!(
        vault.collateral_amount >= amount,
        StableError::InsufficientCollateral
    );

    // Check the ratio after withdrawal
    let new_collateral = vault.collateral_amount - amount;
    if vault.debt_amount > 0 {
        let temp_vault = Vault {
            owner: vault.owner,
            collateral_amount: new_collateral,
            debt_amount: vault.debt_amount,
            last_interest_accrual: vault.last_interest_accrual,
            bump: vault.bump,
        };
        let ratio = temp_vault.collateral_ratio_bps(config.oracle_price);
        require!(
            ratio >= config.collateral_ratio_bps,
            StableError::WithdrawalBreachesRatio
        );
    }

    // Transfer SOL from collateral vault PDA to user
    **ctx.accounts.collateral_vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += amount;

    vault.collateral_amount = new_collateral;
    config.total_collateral = config.total_collateral.saturating_sub(amount);

    emit!(CollateralWithdrawn {
        owner: ctx.accounts.owner.key(),
        amount,
        new_collateral_total: vault.collateral_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
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
