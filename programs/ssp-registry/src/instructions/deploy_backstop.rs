use anchor_lang::prelude::*;

use crate::error::SspError;
use crate::state::{PoolConfig, RegimeState, ReserveVault};

#[derive(Accounts)]
pub struct DeployBackstop<'info> {
    pub authority: Signer<'info>,

    #[account(seeds = [b"pool", pool.base_mint.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, PoolConfig>,

    #[account(seeds = [b"regime", pool.key().as_ref()], bump = regime.bump)]
    pub regime: Account<'info, RegimeState>,

    #[account(mut, seeds = [b"reserve", pool.base_mint.as_ref()], bump = reserve.bump)]
    pub reserve: Account<'info, ReserveVault>,
}

pub fn handler(ctx: Context<DeployBackstop>, amount: u64) -> Result<()> {
    require!(amount <= ctx.accounts.reserve.free(), SspError::InsufficientReserve);
    require!(ctx.accounts.regime.outcome == 3, SspError::NotEligible);

    let reserve = &mut ctx.accounts.reserve;
    reserve.deployed_lamports = reserve.deployed_lamports.saturating_add(amount);
    Ok(())
}
