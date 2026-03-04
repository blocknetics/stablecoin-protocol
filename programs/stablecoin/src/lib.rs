use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("3g5ajnYHrjjPDDJtfRFp54EJQsbC4V1QKX7ncgMUEnwN");

/// Stablecoin Protocol — Collateral-backed stablecoin on Solana
///
/// Features:
/// - Multi-collateral vaults (SOL → USD stablecoin)
/// - Peg Stability Module (USDC ↔ stablecoin 1:1)
/// - Liquidation auctions with bonus
/// - Flash minting
/// - Governance-controlled interest rates
/// - Emergency shutdown
#[program]
pub mod stablecoin {
    use super::*;

    /// Initialize the protocol with configuration parameters.
    pub fn initialize(
        ctx: Context<Initialize>,
        collateral_ratio_bps: u64,
        liquidation_ratio_bps: u64,
        liquidation_bonus_bps: u64,
        stability_fee_bps: u64,
        psm_fee_bps: u64,
        flash_mint_fee_bps: u64,
        initial_oracle_price: u64,
    ) -> Result<()> {
        instructions::initialize::handler(
            ctx,
            collateral_ratio_bps,
            liquidation_ratio_bps,
            liquidation_bonus_bps,
            stability_fee_bps,
            psm_fee_bps,
            flash_mint_fee_bps,
            initial_oracle_price,
        )
    }

    /// Open a new collateral vault, deposit SOL, and mint stablecoins.
    pub fn open_vault(
        ctx: Context<OpenVault>,
        collateral_amount: u64,
        mint_amount: u64,
    ) -> Result<()> {
        instructions::open_vault::handler(ctx, collateral_amount, mint_amount)
    }

    /// Close a vault: repay all debt and reclaim collateral.
    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        instructions::close_vault::handler(ctx)
    }

    /// Deposit additional SOL collateral into an existing vault.
    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        instructions::deposit_collateral::handler(ctx, amount)
    }

    /// Withdraw excess collateral while maintaining the minimum ratio.
    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        instructions::withdraw_collateral::handler(ctx, amount)
    }

    /// Liquidate an under-collateralized vault.
    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        instructions::liquidate::handler(ctx)
    }

    /// Flash mint stablecoins (borrow and repay within same transaction).
    pub fn flash_mint(ctx: Context<FlashMint>, amount: u64) -> Result<()> {
        instructions::flash_mint::handler(ctx, amount)
    }

    /// PSM: Swap USDC in for stablecoins (1:1 minus fee).
    pub fn psm_swap_in(ctx: Context<PsmSwapIn>, usdc_amount: u64) -> Result<()> {
        instructions::psm::handler_swap_in(ctx, usdc_amount)
    }

    /// PSM: Swap stablecoins out for USDC (1:1 minus fee).
    pub fn psm_swap_out(ctx: Context<PsmSwapOut>, stablecoin_amount: u64) -> Result<()> {
        instructions::psm::handler_swap_out(ctx, stablecoin_amount)
    }

    /// Governance: Update the annual stability fee.
    pub fn update_interest_rate(ctx: Context<UpdateInterestRate>, new_rate_bps: u64) -> Result<()> {
        instructions::governance::handler_update_rate(ctx, new_rate_bps)
    }

    /// Governance: Update the oracle price feed.
    pub fn update_oracle_price(ctx: Context<UpdateInterestRate>, new_price: u64) -> Result<()> {
        instructions::governance::handler_update_oracle(ctx, new_price)
    }

    /// Emergency: Activate emergency shutdown.
    pub fn emergency_shutdown(ctx: Context<EmergencyShutdown>) -> Result<()> {
        instructions::emergency::handler(ctx)
    }
}
