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
    positions.sort_by(|a, b| {
        a.claim_id
            .cmp(&b.claim_id)
            .then(a.agent_id.cmp(&b.agent_id))
            .then(a.stance.as_token().cmp(b.stance.as_token()))
    });
    let positions: Vec<Json> = positions
        .iter()
        .map(|p| {
            let cites: Vec<Json> = p
                .citations
                .iter()
                .map(|c| {
                    json::obj(vec![
                        ("source", json::s(&c.source)),
                        ("locator", json::s(&c.locator)),
                    ])
                })
                .collect();
            json::obj(vec![
                ("agent", json::s(&p.agent_id)),
                ("claim", json::s(&p.claim_id)),
                ("stance", json::s(p.stance.as_token())),
                ("confidence", json::num(p.confidence)),
                ("note", json::s(&p.note)),
                ("citations", Json::Arr(cites)),
            ])
        })
        .collect();

    let verdicts: Vec<Json> = adj.verdicts.values().map(verdict_json).collect();

    json::obj(vec![
        ("schema", json::s("quorumforge.bundle.v1")),
        ("deliberation_id", json::s(&delib.id)),
        ("question", json::s(&delib.question)),
        ("policy", policy_json(&adj.policy)),
        ("agents", Json::Arr(agents)),
        ("claims", Json::Arr(claims)),
        ("positions", Json::Arr(positions)),
        ("verdicts", Json::Arr(verdicts)),
        ("summary", summary_json(adj)),
    ])
}

fn policy_json(policy: &crate::adjudicate::Policy) -> Json {
    json::obj(vec![
        ("consensus_threshold", json::num(policy.consensus_threshold)),
        ("dissent_ceiling", json::num(policy.dissent_ceiling)),
        ("minimum_mass", json::num(policy.minimum_mass)),
    ])
}

fn summary_json(adj: &Adjudication) -> Json {
    json::obj(vec![
        ("consensus", json::num(adj.tally.consensus as f64)),
        ("contested", json::num(adj.tally.contested as f64)),
        ("split", json::num(adj.tally.split as f64)),
        ("unsupported", json::num(adj.tally.unsupported as f64)),
        ("total_claims", json::num(adj.tally.total() as f64)),
        ("cohesion", json::num(round6(adj.cohesion))),
    ])
}

fn verdict_json(v: &Verdict) -> Json {
    json::obj(vec![
        ("claim", json::s(&v.claim_id)),
        ("normalized", json::s(&v.normalized)),
        ("outcome", json::s(v.outcome.as_token())),
        ("affirmed", Json::Bool(v.affirmed)),
        ("support_mass", json::num(round6(v.support_mass))),
        (
            "contradiction_mass",
            json::num(round6(v.contradiction_mass)),
        ),
        ("decisive_mass", json::num(round6(v.decisive_mass))),
        ("polarity", json::num(round6(v.polarity))),
        ("dissent_ratio", json::num(round6(v.dissent_ratio))),
        ("supporters", json::num(v.supporters as f64)),
