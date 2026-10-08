use anchor_lang::prelude::*;

use crate::error::SspError;
use crate::state::{PoolConfig, Regime, RegimeState};

#[derive(Accounts)]
pub struct SettleRegime<'info> {
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, PoolConfig>,

    #[account(mut, seeds = [b"regime", pool.key().as_ref()], bump = regime.bump)]
    pub regime: Account<'info, RegimeState>,
}

pub fn handler(
    ctx: Context<SettleRegime>,
    outcome: u8,
    polarity_bps: i32,
    stress_bps: u16,
) -> Result<()> {
    require!((-10_000..=10_000).contains(&polarity_bps), SspError::BadPolarity);
    require!(stress_bps <= 5_000, SspError::BadPolarity);
    let kind = Regime::from_u8(outcome)?;

    let regime = &mut ctx.accounts.regime;
    regime.epoch += 1;
    regime.outcome = outcome;
    regime.polarity_bps = polarity_bps;
    regime.stress_bps = stress_bps;
    regime.stability_index_bps = polarity_bps.unsigned_abs() as u16;
    regime.settled_at = Clock::get()?.slot;

    // Expand relaxes the fee, Hedge tightens it. The band is intentionally
    // narrow so a pool is never repriced more than a few percent per epoch.
    let pool = &mut ctx.accounts.pool;
    pool.fee_bps = match kind {
        Regime::Expand => pool.fee_bps.saturating_sub(5).max(PoolConfig::FEE_FLOOR),
        Regime::Hedge => pool.fee_bps.saturating_add(10).min(PoolConfig::FEE_CEIL),
        Regime::Watch | Regime::Backstop => pool.fee_bps,
    };
    Ok(())
}
