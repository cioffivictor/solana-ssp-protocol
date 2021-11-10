//! Tests for deterministic bundles, normalization, and end-to-end runs against
//! the checked-in sample deliberations.

use quorumforge::adjudicate::{adjudicate, Outcome, Policy};
use quorumforge::normalize::{normalize_text, similarity};
use quorumforge::{bundle, normalize, parse};

const CACHE: &str = include_str!("../samples/cache-coherence.qf");
const MIGRATION: &str = include_str!("../samples/migration-strategy.json");
const NORM: &str = include_str!("../samples/normalization.qf");

fn run(path: &str, src: &str) -> (quorumforge::Deliberation, quorumforge::Adjudication) {
    let mut d = parse::parse_auto(path, src).unwrap();
    normalize::normalize_deliberation(&mut d);
    let adj = adjudicate(&d, &Policy::default());
    (d, adj)
}

#[test]
fn bundle_is_byte_deterministic() {
    let (d, adj) = run("cache.qf", CACHE);
    let b1 = bundle::build(&d, &adj);
    let b2 = bundle::build(&d, &adj);
    assert_eq!(bundle::to_json(&b1), bundle::to_json(&b2));
    assert_eq!(b1.digest, b2.digest);
}

#[test]
fn bundle_digest_survives_reparse() {
    let (d, adj) = run("cache.qf", CACHE);
    let b = bundle::build(&d, &adj);
    assert!(bundle::verify(&b), "freshly built bundle must verify");

