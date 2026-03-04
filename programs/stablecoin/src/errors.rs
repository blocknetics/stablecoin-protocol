use anchor_lang::prelude::*;

/// Custom errors for the Stablecoin Protocol.
#[error_code]
pub enum StableError {
    #[msg("Vault collateral ratio is below the minimum required")]
    BelowCollateralRatio,

    #[msg("Vault is not eligible for liquidation")]
    VaultNotLiquidatable,

    #[msg("Flash-minted tokens were not fully repaid")]
    FlashMintNotRepaid,

    #[msg("Protocol is in emergency shutdown mode")]
    ProtocolShutdown,

    #[msg("Caller is not authorized for this operation")]
    Unauthorized,

    #[msg("Arithmetic overflow or underflow")]
    MathOverflow,

    #[msg("Insufficient collateral in the vault")]
    InsufficientCollateral,

    #[msg("Insufficient debt to perform this operation")]
    InsufficientDebt,

    #[msg("Vault still has outstanding debt")]
    VaultHasDebt,

    #[msg("Withdrawal would breach the collateral ratio")]
    WithdrawalBreachesRatio,

    #[msg("PSM reserve is insufficient for this swap")]
    InsufficientPsmReserve,

    #[msg("Invalid parameter value")]
    InvalidParameter,

    #[msg("Flash mint amount must be greater than zero")]
    ZeroFlashMint,
}
