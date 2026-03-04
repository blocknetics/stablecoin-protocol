use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Flash mint: borrow stablecoins, execute logic, repay + fee in same tx.
/// The flash-minted amount is minted, then the user must have burned
/// (amount + fee) before the instruction ends.
pub fn handler(ctx: Context<FlashMint>, amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.is_shutdown, StableError::ProtocolShutdown);
    require!(amount > 0, StableError::ZeroFlashMint);

    let fee = (amount as u128)
        .checked_mul(config.flash_mint_fee_bps as u128)
        .unwrap()
        / 10_000;
    let fee = fee as u64;
    let repay_amount = amount.checked_add(fee).ok_or(StableError::MathOverflow)?;

    // Mint the flash amount to the borrower
    let config_seeds: &[&[u8]] = &[b"config", &[config.bump]];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                to: ctx.accounts.borrower_stablecoin_account.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            &[config_seeds],
        ),
        amount,
    )?;

    // The borrower must burn (amount + fee) from their account
    // In a real protocol this would involve a CPI callback, but for
    // this implementation we enforce the burn in the same instruction.
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                from: ctx.accounts.borrower_stablecoin_account.to_account_info(),
                authority: ctx.accounts.borrower.to_account_info(),
            },
        ),
        repay_amount,
    )?;

    emit!(FlashMinted {
        borrower: ctx.accounts.borrower.key(),
        amount,
        fee,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct FlashMint<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    #[account(
        mut,
        seeds = [b"stablecoin-mint"],
        bump = config.mint_bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// Borrower's stablecoin token account
    #[account(mut)]
    pub borrower_stablecoin_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
