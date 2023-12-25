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
            let boundary = rest
                .chars()
                .next()
                .map(|c| !c.is_alphanumeric())
                .unwrap_or(true);
            if !boundary {
                continue;
            }
            // A hedge may be followed by a comma or other punctuation
            // ("arguably, ...") which should also be dropped.
            let rest = rest.trim_start_matches(|c: char| c == ',' || c == ':' || c.is_whitespace());
            // Only strip when a hedge is followed by more content.
            if !rest.is_empty() {
                joined = rest.to_string();
                break;
            }
        }
    }

    // 4. Fold a few common contractions to their expanded forms so that
    //    "isn't" and "is not" normalise together.
    let expansions: &[(&str, &str)] = &[
        ("isn't", "is not"),
        ("aren't", "are not"),
        ("wasn't", "was not"),
        ("weren't", "were not"),
        ("doesn't", "does not"),
        ("don't", "do not"),
        ("didn't", "did not"),
        ("can't", "cannot"),
        ("won't", "will not"),
        ("shouldn't", "should not"),
        ("couldn't", "could not"),
        ("wouldn't", "would not"),
        ("it's", "it is"),
    ];
    for (from, to) in expansions {
        if joined.contains(from) {
            joined = joined.replace(from, to);
        }
    }

    // 5. Re-collapse any whitespace introduced by replacement.
    joined.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Populate the `normalized` field of every claim in a deliberation.
pub fn normalize_deliberation(delib: &mut Deliberation) {
    let updates: Vec<(String, String)> = delib
        .claims
        .values()
        .map(|c| (c.id.clone(), normalize_text(&c.text)))
        .collect();
    for (id, normalized) in updates {
        if let Some(claim) = delib.claims.get_mut(&id) {
            claim.normalized = normalized;
        }
    }
}

/// Group claim ids by their normalized text. Claims that share a normalized
/// form are treated as the same proposition for consensus purposes.
///
/// The returned map is ordered by normalized text; each value lists claim ids
/// in sorted order, so the grouping is fully deterministic.
