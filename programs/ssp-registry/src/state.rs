use anchor_lang::prelude::*;

/// Regime classification, mirroring the off-chain classifier in
/// src/adjudicate.rs. Discriminants are stable and are part of the ABI.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Regime {
    Expand,
    Watch,
    Hedge,
    Backstop,
}

impl Regime {
    pub fn from_u8(v: u8) -> Result<Self> {
        match v {
            0 => Ok(Regime::Expand),
            1 => Ok(Regime::Watch),
            2 => Ok(Regime::Hedge),
            3 => Ok(Regime::Backstop),
            _ => err!(crate::error::SspError::BadRegime),
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Regime::Expand => "expand",
            Regime::Watch => "watch",
            Regime::Hedge => "hedge",
            Regime::Backstop => "backstop",
        }
    }
}

/// A supported pool. Seeds: [b"pool", base_mint].
#[account]
#[derive(Default)]
pub struct PoolConfig {
    pub quote_mint: Pubkey,
    pub base_mint: Pubkey,
    /// Base fee in basis points, moved only inside FEE_FLOOR..=FEE_CEIL.
    pub fee_bps: u16,
    /// Depth floor in lamports. Below this the pool is backstop-eligible.
    pub min_depth: u64,
    pub active: bool,
    pub bump: u8,
}

impl PoolConfig {
    pub const LEN: usize = 32 + 32 + 2 + 8 + 1 + 1;
    pub const FEE_FLOOR: u16 = 10;
    pub const FEE_CEIL: u16 = 200;
}

/// Per-pool regime state. Seeds: [b"regime", pool].
#[account]
#[derive(Default)]
pub struct RegimeState {
    pub epoch: u64,
    pub outcome: u8,
    /// Signed polarity in basis points, -10_000..=10_000.
    pub polarity_bps: i32,
    /// Losing-side share in basis points.
    pub stress_bps: u16,
    /// Depth-weighted Stability Index in basis points.
    pub stability_index_bps: u16,
    pub settled_at: u64,
    pub bump: u8,
}

impl RegimeState {
    pub const LEN: usize = 8 + 1 + 4 + 2 + 2 + 8 + 1;
}

/// Reserve capital held for one pool backstop. Seeds: [b"reserve", base_mint].
#[account]
#[derive(Default)]
pub struct ReserveVault {
    pub reserve_lamports: u64,
    pub deployed_lamports: u64,
    pub bump: u8,
}

impl ReserveVault {
    pub const LEN: usize = 8 + 8 + 1;

    pub fn free(&self) -> u64 {
        self.reserve_lamports.saturating_sub(self.deployed_lamports)
    }
}

/// One agent attestation for one epoch.
/// Seeds: [b"attest", pool, agent, epoch_le].
#[account]
#[derive(Default)]
pub struct Attestation {
    pub pool: Pubkey,
    pub agent: Pubkey,
    pub epoch: u64,
    /// 0 = healthy, 1 = stressed, 2 = abstain.
    pub stance: u8,
    pub confidence_bps: u16,
    pub slot: u64,
    pub bump: u8,
}

impl Attestation {
    pub const LEN: usize = 32 + 32 + 8 + 1 + 2 + 8 + 1;
}
