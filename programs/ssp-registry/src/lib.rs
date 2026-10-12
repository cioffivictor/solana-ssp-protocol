//! # SSP registry program
//!
//! The on-chain half of the Solana Stability Protocol. It holds three things:
//! the registry of supported pools, the per-pool regime state, and the reserve
//! vault that funds the liquidity backstop.
//!
//! The protocol splits cleanly in two. Off-chain, the risk kernel turns recorded
//! liquidity-agent attestations into a liquidity regime (expand, watch, hedge,
//! backstop) and a Stability Index. On-chain, this program records that regime,
//! reprices the pool fee inside a narrow band, and lets an authorised caller
//! pull reserve capital into a pool whose depth fell below its floor.
//!
//! The kernel never talks to the chain and the program never re-derives a
//! verdict. That boundary is what makes both halves auditable.

use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

// Placeholder. Run `anchor keys list` after the first build and paste the
// generated id here, into Anchor.toml, and into viewer/src/solana.ts.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ssp_registry {
    use super::*;

    /// Register a supported pool and open its reserve vault.
    pub fn register_pool(ctx: Context<RegisterPool>, fee_bps: u16, min_depth: u64) -> Result<()> {
        instructions::register_pool::handler(ctx, fee_bps, min_depth)
    }

    /// Record one liquidity agent's attestation for the current epoch.
    pub fn submit_attestation(
        ctx: Context<SubmitAttestation>,
        stance: u8,
        confidence_bps: u16,
    ) -> Result<()> {
        instructions::submit_attestation::handler(ctx, stance, confidence_bps)
    }

    /// Write the epoch regime, derived off-chain, and reprice the pool.
    pub fn settle_regime(
        ctx: Context<SettleRegime>,
        outcome: u8,
        polarity_bps: i32,
        stress_bps: u16,
    ) -> Result<()> {
        instructions::settle_regime::handler(ctx, outcome, polarity_bps, stress_bps)
    }

    /// Pull reserve capital into a pool whose depth fell below its floor.
    pub fn deploy_backstop(ctx: Context<DeployBackstop>, amount: u64) -> Result<()> {
        instructions::deploy_backstop::handler(ctx, amount)
    }
}
