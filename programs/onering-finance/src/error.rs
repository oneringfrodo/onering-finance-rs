use anchor_lang::prelude::*;

#[error]
pub enum CommonError {
    #[msg("Access denied")]
    AccessDenied,

    #[msg("Service disabled")]
    ServiceDisabled,
    #[msg("Market locked")]
    MarketLocked,
    #[msg("Reserved account ristricted")]
    ReserveFrozen,

    #[msg("Stable token is invalid")]
    InvalidStableMint,
    #[msg("1USD token is invalid")]
    InvalidOusdMint,

    #[msg("Stable token account owner is invalid")]
    InvalidStableAccountOwner,
    #[msg("1USD token account owner is invalid")]
    InvalidOusdAccountOwner,

    #[msg("Withdrawal amount too much")]
    WithdrawalAmountTooMuch,
    #[msg("Claim amount too much")]
    ClaimAmountTooMuch,
    #[msg("Insufficient stable balance")]
    InsufficientStableBalance,
    #[msg("Insufficient 1USD balance")]
    InsufficientOusdBalance,
    #[msg("Insufficient withdrawal liquidity")]
    InsufficientWithdrawalLiquidity,
}
