//! Deterministic evidence bundles.
//!
//! A *bundle* is a canonical, self-contained snapshot of a deliberation and its
//! adjudication. Bundles are designed to be byte-stable: the same inputs always
//! produce the same bytes, so they can be committed, diffed, and checksummed.
//! A bundle carries its own content digest (a small, dependency-free FNV-1a
//! hash) so downstream tools can detect tampering or drift.

use crate::adjudicate::{Adjudication, Verdict};
use crate::json::{self, Json};
use crate::model::{Deliberation, Position};

/// A packaged deliberation plus verdicts plus an integrity digest.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    pub deliberation: Deliberation,
    pub adjudication: Adjudication,
    /// Hex FNV-1a digest of the canonical JSON body (excluding the digest).
    pub digest: String,
}

/// Build a bundle from a deliberation and its adjudication.
pub fn build(delib: &Deliberation, adj: &Adjudication) -> Bundle {
    let body = canonical_body(delib, adj);
    let text = json::to_string(&body);
    let digest = fnv1a_hex(text.as_bytes());
    Bundle {
        deliberation: delib.clone(),
        adjudication: adj.clone(),
        digest,
    }
}

/// The canonical JSON body of a bundle, with the digest omitted. This exact
/// value is what the digest is computed over, and what [`to_json`] wraps.
fn canonical_body(delib: &Deliberation, adj: &Adjudication) -> Json {
    // Agents in id order (BTreeMap already ordered).
    let agents: Vec<Json> = delib
        .agents
        .values()
        .map(|a| {
            json::obj(vec![
                ("id", json::s(&a.id)),
                ("name", json::s(&a.name)),
                ("role", json::s(&a.role)),
                ("weight", json::num(a.weight)),
            ])
        })
        .collect();

    let claims: Vec<Json> = delib
        .claims
        .values()
        .map(|c| {
            json::obj(vec![
                ("id", json::s(&c.id)),
                ("topic", json::s(&c.topic)),
                ("text", json::s(&c.text)),
                ("normalized", json::s(&c.normalized)),
            ])
        })
        .collect();

    // Positions sorted deterministically by (claim, agent, stance).
    let mut positions: Vec<&Position> = delib.positions.iter().collect();
