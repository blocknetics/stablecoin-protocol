use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Liquidate an under-collateralized vault.
/// Liquidator repays the vault's debt and receives collateral + bonus.
pub fn handler(ctx: Context<Liquidate>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let vault = &mut ctx.accounts.vault;

    // Accrue interest
    vault.accrue_interest(config.stability_fee_bps, Clock::get()?.unix_timestamp);

    // Check vault is below liquidation threshold
    let ratio = vault.collateral_ratio_bps(config.oracle_price);
    require!(
        ratio < config.liquidation_ratio_bps,
        StableError::VaultNotLiquidatable
    );

    let debt = vault.debt_amount;
    let collateral = vault.collateral_amount;

    // Calculate collateral to seize: debt_value + bonus
    // debt is in stablecoin units (6 decimals, pegged to $1)
    // collateral_value_for_debt = debt * 1e9 / oracle_price (convert $ to lamports)
    let collateral_for_debt = (debt as u128)
        .checked_mul(1_000_000_000)
        .unwrap()
        / (config.oracle_price as u128);

    let bonus = (collateral_for_debt as u128)
        .checked_mul(config.liquidation_bonus_bps as u128)
        .unwrap()
        / 10_000;

    let total_seize = std::cmp::min(
        (collateral_for_debt + bonus) as u64,
        collateral,
    );

    let bonus_actual = total_seize.saturating_sub(collateral_for_debt as u64);

    // Burn stablecoins from the liquidator
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                from: ctx.accounts.liquidator_stablecoin_account.to_account_info(),
                authority: ctx.accounts.liquidator.to_account_info(),
            },
        ),
        debt,
    )?;

    // Transfer seized collateral to liquidator
    **ctx.accounts.collateral_vault.to_account_info().try_borrow_mut_lamports()? -= total_seize;
    **ctx.accounts.liquidator.to_account_info().try_borrow_mut_lamports()? += total_seize;

    // Update vault — remaining collateral goes back to owner (if any)
    let remaining_collateral = collateral.saturating_sub(total_seize);

    // If there's remaining collateral, return it to vault owner
    if remaining_collateral > 0 {
        **ctx.accounts.collateral_vault.to_account_info().try_borrow_mut_lamports()? -= remaining_collateral;
        **ctx.accounts.vault_owner.to_account_info().try_borrow_mut_lamports()? += remaining_collateral;
    }

    // Update protocol totals
    config.total_debt = config.total_debt.saturating_sub(debt);
    config.total_collateral = config.total_collateral.saturating_sub(collateral);

    // Zero out the vault
    vault.collateral_amount = 0;
    vault.debt_amount = 0;

    emit!(VaultLiquidated {
        vault_owner: vault.owner,
        liquidator: ctx.accounts.liquidator.key(),
        debt_repaid: debt,
        collateral_seized: total_seize,
        bonus: bonus_actual,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    /// The liquidator who repays the debt
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    /// The under-collateralized vault to liquidate
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
        close = vault_owner,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: The vault owner, receives remaining collateral and rent
    #[account(mut, address = vault.owner)]
    pub vault_owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"stablecoin-mint"],
        bump = config.mint_bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// Liquidator's stablecoin token account (to burn from)
    #[account(mut)]
    pub liquidator_stablecoin_account: Account<'info, TokenAccount>,

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
