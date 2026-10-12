use anchor_lang::prelude::*;

#[error_code]
pub enum SspError {
    #[msg("Unknown regime discriminant")]
    BadRegime,
    #[msg("Unknown stance discriminant")]
    BadStance,
    #[msg("Confidence must be within 0..=10000 basis points")]
    BadConfidence,
    #[msg("Polarity must be within -10000..=10000 basis points")]
    BadPolarity,
    #[msg("Pool is not active")]
    PoolInactive,
    #[msg("Deployment exceeds the free reserve balance")]
    InsufficientReserve,
    #[msg("Pool is not in the backstop regime")]
    NotEligible,
}
