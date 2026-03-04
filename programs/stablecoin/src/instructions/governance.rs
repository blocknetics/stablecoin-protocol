use anchor_lang::prelude::*;

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Update the annual stability fee (interest rate). Authority-gated.
pub fn handler_update_rate(ctx: Context<UpdateInterestRate>, new_rate_bps: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(
        ctx.accounts.authority.key() == config.authority,
        StableError::Unauthorized
    );
    require!(new_rate_bps <= 10_000, StableError::InvalidParameter); // max 100%

    let old_rate = config.stability_fee_bps;
    config.stability_fee_bps = new_rate_bps;

    emit!(InterestRateUpdated {
        authority: ctx.accounts.authority.key(),
        old_rate_bps: old_rate,
        new_rate_bps,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Update the oracle price. Authority-gated.
/// In production this would be a Switchboard/Pyth feed; here, manual for simplicity.
pub fn handler_update_oracle(ctx: Context<UpdateInterestRate>, new_price: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(
        ctx.accounts.authority.key() == config.authority,
        StableError::Unauthorized
    );
    require!(new_price > 0, StableError::InvalidParameter);

    config.oracle_price = new_price;
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateInterestRate<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
}
