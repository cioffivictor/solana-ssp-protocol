//! Constant-product pool maths for SSP.
//!
//! Std-only on purpose: the kernel must be able to size a depth target and a
//! price impact without pulling in a math crate. Every function is pure and
//! every division is guarded.

/// Product invariant x * y, saturating on overflow.
pub fn invariant(reserve_quote: u128, reserve_base: u128) -> u128 {
    reserve_quote.saturating_mul(reserve_base)
}

/// Output of a swap of amount_in against reserve_in, with fee_bps taken from the
/// input. Returns 0 when the pool cannot fill the trade.
pub fn swap_out(reserve_in: u128, reserve_out: u128, amount_in: u128, fee_bps: u16) -> u128 {
    if reserve_in == 0 || reserve_out == 0 || amount_in == 0 {
        return 0;
    }
    let net_in = amount_in.saturating_mul(10_000 - fee_bps as u128) / 10_000;
    let numerator = reserve_out.saturating_mul(net_in);
    let denominator = reserve_in.saturating_add(net_in);
    if denominator == 0 {
        0
    } else {
        numerator / denominator
    }
}

/// Price impact of a trade in basis points, capped at 10_000.
pub fn price_impact_bps(reserve_in: u128, amount_in: u128) -> u16 {
    if reserve_in == 0 {
        return 10_000;
    }
    (amount_in.saturating_mul(10_000) / reserve_in).min(10_000) as u16
}

/// Depth a pool needs so a trade of `notional` moves price by no more than
/// `max_impact_bps`. This is the number the WATCH regime publishes as a target.
pub fn depth_target(notional: u128, max_impact_bps: u16) -> u128 {
    if max_impact_bps == 0 {
        return u128::MAX;
    }
    notional.saturating_mul(10_000) / max_impact_bps as u128
}

/// Fee tier for a regime, clamped to the same band the program enforces.
pub fn fee_for_regime(base_fee_bps: u16, outcome: u8) -> u16 {
    match outcome {
        0 => base_fee_bps.saturating_sub(5).max(10),
        2 => base_fee_bps.saturating_add(10).min(200),
        _ => base_fee_bps,
    }
}
