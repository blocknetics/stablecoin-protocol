use anchor_lang::prelude::*;

// ────────────────────────────────────────────────────────────────
// Protocol Configuration Account
// ────────────────────────────────────────────────────────────────

#[account]
#[derive(InitSpace)]
pub struct ProtocolConfig {
    /// The authority who can update protocol parameters and trigger shutdown
    pub authority: Pubkey,

    /// The stablecoin SPL-token mint (PDA-controlled)
    pub stablecoin_mint: Pubkey,

    /// Minimum collateral ratio in basis points (e.g. 15000 = 150%)
    pub collateral_ratio_bps: u64,

    /// Liquidation threshold in basis points (e.g. 12000 = 120%)
    pub liquidation_ratio_bps: u64,

    /// Liquidation bonus in basis points (e.g. 500 = 5%)
    pub liquidation_bonus_bps: u64,

    /// Annual stability fee in basis points (e.g. 200 = 2%)
    pub stability_fee_bps: u64,

    /// PSM swap fee in basis points (e.g. 10 = 0.1%)
    pub psm_fee_bps: u64,

    /// Flash mint fee in basis points (e.g. 9 = 0.09%)
    pub flash_mint_fee_bps: u64,

    /// Whether the protocol is in emergency shutdown
    pub is_shutdown: bool,

    /// Total outstanding stablecoin debt across all vaults
    pub total_debt: u64,

    /// Total collateral held across all vaults (in lamports)
    pub total_collateral: u64,

    /// Oracle price (SOL/USD) stored as fixed-point with 6 decimals
    /// e.g. 150_000_000 = $150.00
    pub oracle_price: u64,

    /// Bump seed for the config PDA
    pub bump: u8,

    /// Bump seed for the stablecoin mint PDA
    pub mint_bump: u8,

    /// Bump seed for the collateral vault PDA
    pub vault_bump: u8,
}

// ────────────────────────────────────────────────────────────────
// User Vault Account
// ────────────────────────────────────────────────────────────────

#[account]
#[derive(InitSpace)]
pub struct Vault {
    /// Owner of the vault
    pub owner: Pubkey,

    /// SOL collateral deposited (in lamports)
    pub collateral_amount: u64,

    /// Outstanding stablecoin debt
    pub debt_amount: u64,

    /// Last timestamp when interest was accrued
    pub last_interest_accrual: i64,

    /// Bump seed for this vault PDA
    pub bump: u8,
}

// ────────────────────────────────────────────────────────────────
// PSM Reserve Tracking Account
// ────────────────────────────────────────────────────────────────

#[account]
#[derive(InitSpace)]
pub struct PsmReserve {
    /// USDC token account holding PSM reserves
    pub reserve_token_account: Pubkey,

    /// Total USDC deposited via PSM swap-in
    pub total_usdc_reserves: u64,

    /// Total stablecoins minted via PSM
    pub total_stablecoins_issued: u64,

    /// Bump seed for the PSM PDA
    pub bump: u8,
}

// ────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────

impl Vault {
    /// Calculate the current collateral ratio in basis points.
    /// Returns `u64::MAX` if vault has no debt.
    pub fn collateral_ratio_bps(&self, oracle_price: u64) -> u64 {
        if self.debt_amount == 0 {
            return u64::MAX;
        }
        // collateral_value = collateral_amount * oracle_price / 1e9  (lamports to SOL * price)
        // ratio = collateral_value / debt * 10000
        // Combined: (collateral * price * 10000) / (debt * 1e9)
        let numerator = (self.collateral_amount as u128)
            .checked_mul(oracle_price as u128)
            .unwrap()
            .checked_mul(10_000)
            .unwrap();
        let denominator = (self.debt_amount as u128)
            .checked_mul(1_000_000_000)
            .unwrap();
        (numerator / denominator) as u64
    }

    /// Accrue interest on the vault debt.
    /// `stability_fee_bps` is the annual rate in bps.
    /// Uses simple interest: debt += debt * rate * elapsed / (365.25 * 86400) / 10000
    pub fn accrue_interest(&mut self, stability_fee_bps: u64, current_timestamp: i64) {
        if self.debt_amount == 0 || self.last_interest_accrual >= current_timestamp {
            return;
        }
        let elapsed = (current_timestamp - self.last_interest_accrual) as u128;
        let seconds_per_year: u128 = 365 * 24 * 3600 + 6 * 3600; // ~365.25 days
        let interest = (self.debt_amount as u128)
            .checked_mul(stability_fee_bps as u128)
            .unwrap()
            .checked_mul(elapsed)
            .unwrap()
            / seconds_per_year
            / 10_000;
        self.debt_amount = self.debt_amount.checked_add(interest as u64).unwrap_or(self.debt_amount);
        self.last_interest_accrual = current_timestamp;
    }
}
