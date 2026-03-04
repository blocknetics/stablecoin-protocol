use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::state::*;
use crate::events::*;

/// Initialize the stablecoin protocol.
/// Creates the protocol config, stablecoin mint, and collateral vault PDAs.
pub fn handler(
    ctx: Context<Initialize>,
    collateral_ratio_bps: u64,
    liquidation_ratio_bps: u64,
    liquidation_bonus_bps: u64,
    stability_fee_bps: u64,
    psm_fee_bps: u64,
    flash_mint_fee_bps: u64,
    initial_oracle_price: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.stablecoin_mint = ctx.accounts.stablecoin_mint.key();
    config.collateral_ratio_bps = collateral_ratio_bps;
    config.liquidation_ratio_bps = liquidation_ratio_bps;
    config.liquidation_bonus_bps = liquidation_bonus_bps;
    config.stability_fee_bps = stability_fee_bps;
    config.psm_fee_bps = psm_fee_bps;
    config.flash_mint_fee_bps = flash_mint_fee_bps;
    config.is_shutdown = false;
    config.total_debt = 0;
    config.total_collateral = 0;
    config.oracle_price = initial_oracle_price;
    config.bump = ctx.bumps.config;
    config.mint_bump = ctx.bumps.stablecoin_mint;
    config.vault_bump = ctx.bumps.collateral_vault;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The protocol authority (deployer / governance)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Protocol configuration PDA
    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolConfig::INIT_SPACE,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    /// The stablecoin SPL-token mint, controlled by the config PDA
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"stablecoin-mint"],
        bump,
    )]
    pub stablecoin_mint: Account<'info, Mint>,

    /// PDA that holds all SOL collateral
    /// CHECK: This is a PDA that only holds SOL lamports, validated by seeds
    #[account(
        mut,
        seeds = [b"collateral-vault"],
        bump,
    )]
    pub collateral_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
