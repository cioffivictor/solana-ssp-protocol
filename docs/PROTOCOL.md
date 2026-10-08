# SSP protocol reference

Two halves, one boundary.

| Half | Lives in | Responsibility |
| --- | --- | --- |
| Risk kernel | `src/` | Read a recorded epoch, derive one regime per pool assertion. |
| Registry program | `programs/ssp-registry/` | Record the regime, move the reserve, reprice the fee. |

The kernel never reads chain state. The program never re-derives a verdict. A
relayer lands `settle_regime` with the values the kernel produced.

## Pool lifecycle

1. `register_pool` creates `PoolConfig` and `ReserveVault`, and sets the depth
   floor and the base fee.
2. Liquidity agents call `submit_attestation` once per epoch with a stance
   (`0` healthy, `1` stressed, `2` abstain) and a confidence in basis points.
3. Off-chain, the kernel aggregates the attestations into `decisive_depth`,
   `polarity` and `stress_ratio`, and classifies the regime.
4. `settle_regime` writes the outcome, polarity and stress, increments the epoch,
   and moves the fee inside `10..=200` basis points: `expand` shaves 5, `hedge`
   adds 10, `watch` and `backstop` hold.
5. If the regime is `backstop`, `deploy_backstop` pulls reserve capital into the
   pool up to the free balance.

## Accounts

| Account | Seeds | Notes |
| --- | --- | --- |
| `PoolConfig` | `[b"pool", base_mint]` | Fee band, depth floor, active flag. |
| `RegimeState` | `[b"regime", pool]` | Epoch, outcome, polarity, stress, index. |
| `ReserveVault` | `[b"reserve", base_mint]` | `free = reserve - deployed`. |
| `Attestation` | `[b"attest", pool, agent, epoch_le]` | One per agent per epoch. |

## Regimes

| Discriminant | Regime | Action |
| --- | --- | --- |
| 0 | `expand` | Add depth, cut fee, release reserve. |
| 1 | `watch` | Hold allocation, skew fee. |
| 2 | `hedge` | Rebalance inventory, raise fee. |
| 3 | `backstop` | Deploy reserve capital as a floor. |

Discriminants are part of the ABI and match `Regime::token()` in the program and
`REGIMES` in `viewer/src/solana.ts`.
