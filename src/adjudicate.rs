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
