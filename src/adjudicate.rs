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
