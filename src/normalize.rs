//! Claim normalization.
//!
//! Two agents rarely phrase the same proposition identically. The normalizer
//! reduces claim text to a canonical form so that near-duplicate claims can be
//! recognised and, optionally, merged. Normalization is intentionally
//! conservative: it lower-cases, collapses whitespace, strips a small set of
//! leading hedges and trailing punctuation, and folds a handful of common
//! contractions. It does *not* attempt semantic paraphrase detection — that
//! would make verdicts non-deterministic and hard to audit.

use crate::model::{Claim, Deliberation};
use std::collections::BTreeMap;

/// Hedging prefixes that carry no propositional content and are dropped from
/// the head of a claim during normalization.
const LEADING_HEDGES: &[&str] = &[
    "i think that",
    "i think",
    "i believe that",
    "i believe",
    "it seems that",
    "it seems",
    "arguably",
    "presumably",
    "clearly",
    "obviously",
    "in my view",
    "it is likely that",
    "likely",
    "perhaps",
    "maybe",
];

/// Compute the canonical form of a single claim's text.
pub fn normalize_text(text: &str) -> String {
    // 1. Lower-case and split on whitespace to collapse runs.
    let lowered = text.to_lowercase();
    let mut joined = lowered.split_whitespace().collect::<Vec<_>>().join(" ");

    // 2. Strip trailing sentence punctuation.
    while matches!(
        joined.chars().last(),
        Some('.') | Some('!') | Some('?') | Some(';')
    ) {
        joined.pop();
    }
    joined = joined.trim().to_string();

    // 3. Drop a single leading hedge if present (longest match first).
    for hedge in LEADING_HEDGES {
        if let Some(rest) = joined.strip_prefix(hedge) {
            // Require a word boundary so "likely" does not swallow "likelihood".
