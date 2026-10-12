use anchor_lang::prelude::*;

use crate::state::{PoolConfig, ReserveVault};

#[derive(Accounts)]
pub struct RegisterPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + PoolConfig::LEN,
        seeds = [b"pool", base_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, PoolConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + ReserveVault::LEN,
        seeds = [b"reserve", base_mint.key().as_ref()],
        bump
    )]
    pub reserve: Account<'info, ReserveVault>,

    /// CHECK: mint of the memecoin, not deserialized here.
    pub base_mint: UncheckedAccount<'info>,
    /// CHECK: mint of the quoted asset, not deserialized here.
    pub quote_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterPool>, fee_bps: u16, min_depth: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.quote_mint = ctx.accounts.quote_mint.key();
    pool.base_mint = ctx.accounts.base_mint.key();
    pool.fee_bps = fee_bps.clamp(PoolConfig::FEE_FLOOR, PoolConfig::FEE_CEIL);
    pool.min_depth = min_depth;
    pool.active = true;
    pool.bump = ctx.bumps.pool;
    ctx.accounts.reserve.bump = ctx.bumps.reserve;
    Ok(())
}
