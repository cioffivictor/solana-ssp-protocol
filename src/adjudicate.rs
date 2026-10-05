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
            return Err(format!(
                "consensus_threshold must be in [0,1], got {}",
                self.consensus_threshold
            ));
        }
        if !(0.0..=1.0).contains(&self.dissent_ceiling) {
            return Err(format!(
                "dissent_ceiling must be in [0,1], got {}",
                self.dissent_ceiling
            ));
        }
        if self.minimum_mass < 0.0 {
            return Err(format!(
                "minimum_mass must be non-negative, got {}",
                self.minimum_mass
            ));
        }
        Ok(())
    }
}

/// The classification of a single claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Strong, low-dissent agreement. See [`Verdict::affirmed`] for direction.
    Consensus,
    /// Meaningful weight on both sides; the council is genuinely divided.
    Contested,
    /// Leaning but below the consensus bar without qualifying as contested.
    Split,
    /// Too little decisive weight to conclude anything.
    Unsupported,
}

impl Outcome {
    /// A stable lowercase token for serialisation.
    pub fn as_token(self) -> &'static str {
        match self {
            Outcome::Consensus => "consensus",
            Outcome::Contested => "contested",
            Outcome::Split => "split",
            Outcome::Unsupported => "unsupported",
        }
    }
}

/// A per-claim verdict with all intermediate quantities retained for auditing.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub claim_id: String,
    pub normalized: String,
    pub outcome: Outcome,
    /// `true` when a consensus/split leans toward the claim being true.
    pub affirmed: bool,
    /// Sum of support votes.
    pub support_mass: f64,
    /// Sum of contradiction vote magnitudes.
    pub contradiction_mass: f64,
    /// `support_mass + contradiction_mass`.
    pub decisive_mass: f64,
    /// `(support_mass - contradiction_mass) / decisive_mass`, or `0` if no mass.
    pub polarity: f64,
    /// Losing-side fraction of decisive mass in `[0, 0.5]`.
    pub dissent_ratio: f64,
    /// Count of supporting positions (non-abstaining).
    pub supporters: usize,
    /// Count of contradicting positions.
    pub dissenters: usize,
    /// Count of abstentions recorded.
    pub abstentions: usize,
    /// Number of distinct citations attached to any position on this claim.
    pub citation_count: usize,
    /// Agent ids on the winning side, sorted, for the report's roster.
    pub majority_agents: Vec<String>,
    /// Agent ids on the losing side, sorted.
    pub minority_agents: Vec<String>,
}

/// The full result of adjudicating a deliberation.
#[derive(Debug, Clone, PartialEq)]
pub struct Adjudication {
    pub deliberation_id: String,
    pub question: String,
    pub policy: Policy,
    /// Verdicts keyed by claim id (ordered) for stable iteration.
    pub verdicts: BTreeMap<String, Verdict>,
    /// Count of each outcome, for headline summaries.
    pub tally: Tally,
    /// A single scalar in `[0,1]` summarising how settled the deliberation is.
    pub cohesion: f64,
}

/// Outcome counts across all claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    pub consensus: usize,
    pub contested: usize,
    pub split: usize,
    pub unsupported: usize,
}

impl Tally {
    pub fn total(&self) -> usize {
        self.consensus + self.contested + self.split + self.unsupported
    }
}

/// Adjudicate a single claim under a policy.
pub fn verdict_for(delib: &Deliberation, claim_id: &str, policy: &Policy) -> Verdict {
    let claim = delib.claims.get(claim_id);
    let normalized = claim
        .map(|c| {
            if c.normalized.is_empty() {
                crate::normalize::normalize_text(&c.text)
            } else {
                c.normalized.clone()
            }
        })
        .unwrap_or_default();

    let mut support_mass = 0.0f64;
    let mut contradiction_mass = 0.0f64;
    let mut supporters = 0usize;
    let mut dissenters = 0usize;
    let mut abstentions = 0usize;
    let mut citation_count = 0usize;
    let mut support_agents: Vec<String> = Vec::new();
    let mut contradict_agents: Vec<String> = Vec::new();

    for pos in delib.positions_for(claim_id) {
        citation_count += pos.citations.len();
        let weight = delib.agent_weight(&pos.agent_id);
        // Clamp confidence defensively so malformed inputs cannot skew a verdict.
        let conf = pos.confidence.clamp(0.0, 1.0);
        let vote = weight * conf;
        match pos.stance {
            Stance::Support => {
                support_mass += vote;
                supporters += 1;
                support_agents.push(pos.agent_id.clone());
            }
            Stance::Contradict => {
                contradiction_mass += vote;
                dissenters += 1;
                contradict_agents.push(pos.agent_id.clone());
            }
            Stance::Abstain => {
                abstentions += 1;
            }
        }
    }

    let decisive_mass = support_mass + contradiction_mass;
    let polarity = if decisive_mass > 0.0 {
        (support_mass - contradiction_mass) / decisive_mass
    } else {
        0.0
    };
    let dissent_ratio = if decisive_mass > 0.0 {
        support_mass.min(contradiction_mass) / decisive_mass
    } else {
        0.0
    };
    let affirmed = polarity >= 0.0;

    let outcome = classify(decisive_mass, polarity, dissent_ratio, policy);

    // The winning side depends on polarity; ties (polarity == 0) treat support
    // as the nominal majority for a stable, documented tie-break.
    let (mut majority_agents, mut minority_agents) = if affirmed {
        (support_agents, contradict_agents)
    } else {
        (contradict_agents, support_agents)
    };
    majority_agents.sort();
    majority_agents.dedup();
    minority_agents.sort();
    minority_agents.dedup();

    Verdict {
        claim_id: claim_id.to_string(),
        normalized,
        outcome,
        affirmed,
        support_mass,
        contradiction_mass,
        decisive_mass,
        polarity,
        dissent_ratio,
        supporters,
        dissenters,
        abstentions,
        citation_count,
        majority_agents,
        minority_agents,
    }
}

/// Apply the policy thresholds to classify one claim.
fn classify(mass: f64, polarity: f64, dissent_ratio: f64, policy: &Policy) -> Outcome {
    if mass <= policy.minimum_mass {
        return Outcome::Unsupported;
    }
    let strong = polarity.abs() >= policy.consensus_threshold;
    if strong && dissent_ratio < policy.dissent_ceiling {
        Outcome::Consensus
    } else if dissent_ratio >= policy.dissent_ceiling {
        Outcome::Contested
    } else {
        Outcome::Split
    }
}

/// Adjudicate a whole deliberation.
pub fn adjudicate(delib: &Deliberation, policy: &Policy) -> Adjudication {
    let mut verdicts = BTreeMap::new();
    let mut tally = Tally::default();
    let mut weighted_cohesion_num = 0.0f64;
    let mut weighted_cohesion_den = 0.0f64;

    for claim_id in delib.claims.keys() {
        let verdict = verdict_for(delib, claim_id, policy);
        match verdict.outcome {
            Outcome::Consensus => tally.consensus += 1,
            Outcome::Contested => tally.contested += 1,
            Outcome::Split => tally.split += 1,
            Outcome::Unsupported => tally.unsupported += 1,
        }
        // Cohesion weights each claim by its decisive mass: heavily-attended
        // claims move the needle more than sparsely-attended ones. A claim's
        // per-claim settledness is `|polarity|` (1 = unanimous, 0 = perfectly
        // split). Unsupported claims contribute no mass and no settledness.
        weighted_cohesion_num += verdict.polarity.abs() * verdict.decisive_mass;
        weighted_cohesion_den += verdict.decisive_mass;
        verdicts.insert(claim_id.clone(), verdict);
    }

    let cohesion = if weighted_cohesion_den > 0.0 {
        weighted_cohesion_num / weighted_cohesion_den
    } else {
        0.0
    };

    Adjudication {
        deliberation_id: delib.id.clone(),
        question: delib.question.clone(),
        policy: *policy,
        verdicts,
        tally,
        cohesion,
    }
}
