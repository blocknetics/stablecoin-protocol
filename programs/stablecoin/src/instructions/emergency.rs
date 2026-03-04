use anchor_lang::prelude::*;

use crate::errors::StableError;
use crate::events::*;
use crate::state::*;

/// Activate emergency shutdown. Authority-gated.
/// Halts all minting operations and allows proportional collateral redemption.
pub fn handler(ctx: Context<EmergencyShutdown>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(
        ctx.accounts.authority.key() == config.authority,
        StableError::Unauthorized
    );
    require!(!config.is_shutdown, StableError::ProtocolShutdown);

    config.is_shutdown = true;

    emit!(EmergencyShutdownActivated {
        authority: ctx.accounts.authority.key(),
        total_debt_at_shutdown: config.total_debt,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct EmergencyShutdown<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
}
