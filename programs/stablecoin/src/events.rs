use anchor_lang::prelude::*;

// ────────────────────────────────────────────────────────────────
// Protocol Events
// ────────────────────────────────────────────────────────────────

#[event]
pub struct VaultOpened {
    pub owner: Pubkey,
    pub collateral_amount: u64,
    pub debt_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultClosed {
    pub owner: Pubkey,
    pub collateral_returned: u64,
    pub debt_repaid: u64,
    pub timestamp: i64,
}

#[event]
pub struct CollateralDeposited {
    pub owner: Pubkey,
    pub amount: u64,
    pub new_collateral_total: u64,
    pub timestamp: i64,
}

#[event]
pub struct CollateralWithdrawn {
    pub owner: Pubkey,
    pub amount: u64,
    pub new_collateral_total: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultLiquidated {
    pub vault_owner: Pubkey,
    pub liquidator: Pubkey,
    pub debt_repaid: u64,
    pub collateral_seized: u64,
    pub bonus: u64,
    pub timestamp: i64,
}

#[event]
pub struct FlashMinted {
    pub borrower: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct PsmSwapped {
    pub user: Pubkey,
    pub direction: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct InterestRateUpdated {
    pub authority: Pubkey,
    pub old_rate_bps: u64,
    pub new_rate_bps: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyShutdownActivated {
    pub authority: Pubkey,
    pub total_debt_at_shutdown: u64,
    pub timestamp: i64,
}
