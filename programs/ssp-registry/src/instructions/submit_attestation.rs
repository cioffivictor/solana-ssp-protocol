use anchor_lang::prelude::*;

use crate::error::SspError;
use crate::state::{Attestation, PoolConfig, RegimeState};

#[derive(Accounts)]
pub struct SubmitAttestation<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,

    #[account(seeds = [b"pool", pool.base_mint.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, PoolConfig>,

    #[account(mut, seeds = [b"regime", pool.key().as_ref()], bump = regime.bump)]
    pub regime: Account<'info, RegimeState>,

    #[account(
        init_if_needed,
        payer = agent,
        space = 8 + Attestation::LEN,
        seeds = [b"attest", pool.key().as_ref(), agent.key().as_ref(), &regime.epoch.to_le_bytes()],
        bump
    )]
    pub attestation: Account<'info, Attestation>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SubmitAttestation>, stance: u8, confidence_bps: u16) -> Result<()> {
    require!(stance <= 2, SspError::BadStance);
    require!(confidence_bps <= 10_000, SspError::BadConfidence);
    require!(ctx.accounts.pool.active, SspError::PoolInactive);

    let att = &mut ctx.accounts.attestation;
    att.pool = ctx.accounts.pool.key();
    att.agent = ctx.accounts.agent.key();
    att.epoch = ctx.accounts.regime.epoch;
    att.stance = stance;
    att.confidence_bps = confidence_bps;
    att.slot = Clock::get()?.slot;
    att.bump = ctx.bumps.attestation;
    Ok(())
}
