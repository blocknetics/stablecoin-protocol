use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// PSM Swap In: Deposit USDC, receive stablecoins 1:1 (minus fee).
pub fn handler_swap_in(ctx: Context<PsmSwapIn>, usdc_amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.is_shutdown, StableError::ProtocolShutdown);

    let fee = (usdc_amount as u128)
        .checked_mul(config.psm_fee_bps as u128)
        .unwrap()
        / 10_000;
    let fee = fee as u64;
    let mint_amount = usdc_amount.saturating_sub(fee);

    // Transfer USDC from user to PSM reserve
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc_account.to_account_info(),
                to: ctx.accounts.psm_usdc_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        usdc_amount,
    )?;

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

    // Update PSM reserves
    let psm = &mut ctx.accounts.psm_reserve;
    psm.total_usdc_reserves = psm.total_usdc_reserves.checked_add(usdc_amount).ok_or(StableError::MathOverflow)?;
    psm.total_stablecoins_issued = psm.total_stablecoins_issued.checked_add(mint_amount).ok_or(StableError::MathOverflow)?;

    config.total_debt = config.total_debt.checked_add(mint_amount).ok_or(StableError::MathOverflow)?;

    emit!(PsmSwapped {
        user: ctx.accounts.user.key(),
        direction: "swap_in".to_string(),
        amount_in: usdc_amount,
        amount_out: mint_amount,
        fee,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// PSM Swap Out: Burn stablecoins, receive USDC 1:1 (minus fee).
pub fn handler_swap_out(ctx: Context<PsmSwapOut>, stablecoin_amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.is_shutdown, StableError::ProtocolShutdown);

    let fee = (stablecoin_amount as u128)
        .checked_mul(config.psm_fee_bps as u128)
        .unwrap()
        / 10_000;
    let fee = fee as u64;
    let usdc_out = stablecoin_amount.saturating_sub(fee);

    let psm = &mut ctx.accounts.psm_reserve;
    require!(
        psm.total_usdc_reserves >= usdc_out,
        StableError::InsufficientPsmReserve
    );

    // Burn stablecoins from user
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                from: ctx.accounts.user_stablecoin_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        stablecoin_amount,
    )?;

    // Transfer USDC from PSM reserve to user (PDA-signed)
    let config_seeds: &[&[u8]] = &[b"config", &[config.bump]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.psm_usdc_account.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            &[config_seeds],
        ),
        usdc_out,
    )?;

    // Update PSM reserves
    psm.total_usdc_reserves = psm.total_usdc_reserves.saturating_sub(usdc_out);
    psm.total_stablecoins_issued = psm.total_stablecoins_issued.saturating_sub(stablecoin_amount);

    config.total_debt = config.total_debt.saturating_sub(stablecoin_amount);

    emit!(PsmSwapped {
        user: ctx.accounts.user.key(),
        direction: "swap_out".to_string(),
        amount_in: stablecoin_amount,
        amount_out: usdc_out,
        fee,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// ── Account Contexts ────────────────────────────────────────────

#[derive(Accounts)]
pub struct PsmSwapIn<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    #[account(
        mut,
        seeds = [b"psm-reserve"],
        bump = psm_reserve.bump,
    )]
    pub psm_reserve: Account<'info, PsmReserve>,

    #[account(
        mut,
        seeds = [b"stablecoin-mint"],
        bump = config.mint_bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// User's USDC token account (source)
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,

    /// User's stablecoin token account (destination)
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// PSM's USDC reserve token account
    #[account(mut)]
    pub psm_usdc_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PsmSwapOut<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    #[account(
        mut,
        seeds = [b"psm-reserve"],
        bump = psm_reserve.bump,
    )]
    pub psm_reserve: Account<'info, PsmReserve>,

    #[account(
        mut,
        seeds = [b"stablecoin-mint"],
        bump = config.mint_bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// User's stablecoin token account (source — to burn)
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// User's USDC token account (destination)
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,

    /// PSM's USDC reserve token account
    #[account(mut)]
    pub psm_usdc_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
