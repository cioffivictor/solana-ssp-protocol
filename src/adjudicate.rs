//! The adjudication engine.
//!
//! Given a normalized [`Deliberation`], the engine computes a [`Verdict`] for
//! every claim and rolls those up into a [`Adjudication`] for the whole
//! deliberation. The scoring model is deliberately simple and fully explained
//! here so that any verdict can be re-derived by hand.
//!
//! ## Scoring
//!
//! For a claim `c`, each non-abstaining position contributes a *signed vote*:
//!
//! ```text
//! vote = stance.sign() * agent_weight * confidence
//! ```
//!
//! where `stance.sign()` is `+1` for support and `-1` for contradiction. Let:
//!
//! * `S` = sum of support votes (a non-negative number)
//! * `C` = sum of the magnitudes of contradiction votes (non-negative)
//! * `mass` = `S + C` (total decisive weight on the claim)
//!
//! The **net score** is `S - C`, and the **polarity** is `net / mass` in
//! `[-1, 1]`. A claim with no decisive positions has `mass == 0` and is
//! classified `Unsupported`.
//!
//! ## Verdict classification
//!
//! Two thresholds govern classification, both configurable:
//!
//! * `consensus_threshold` (default `0.66`): if `|polarity| >=` this and the
//!   dissent ratio is below `dissent_ceiling`, the claim is a
//!   [`Outcome::Consensus`] (affirmed if polarity is positive, negated if
//!   negative).
//! * `dissent_ceiling` (default `0.34`): the fraction of decisive mass on the
//!   losing side. When both sides carry meaningful weight, the claim is
//!   [`Outcome::Contested`].
//!
//! Claims that clear neither bar land in [`Outcome::Split`]. This gives four
//! mutually exclusive outcomes: `Consensus`, `Contested`, `Split`, and
//! `Unsupported`.

use crate::model::{Deliberation, Stance};
use std::collections::BTreeMap;

/// Tunable thresholds for the classifier. Defaults are chosen so that a
/// two-thirds supermajority with limited dissent reads as consensus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Policy {
    /// Minimum absolute polarity for a `Consensus` outcome.
    pub consensus_threshold: f64,
    /// Maximum losing-side mass fraction tolerated within a `Consensus`.
    pub dissent_ceiling: f64,
    /// Minimum decisive mass required to escape `Unsupported`.
    pub minimum_mass: f64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            consensus_threshold: 0.66,
            dissent_ceiling: 0.34,
            minimum_mass: 1e-9,
        }
    }
}

impl Policy {
    /// Validate a policy, returning a human-readable error on nonsense inputs.
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.consensus_threshold) {
