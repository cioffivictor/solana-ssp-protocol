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

    // Serialise, strip the digest field the way the CLI's `verify` does, and
    // recompute. This guards the idempotent-number invariant end to end.
    let out = bundle::to_json(&b);
    let root = quorumforge::json::parse(&out).unwrap();
    let entries = root.as_object().unwrap();
    let stored = root.get("digest").unwrap().as_str().unwrap();
    let body: Vec<(String, quorumforge::json::Json)> = entries
        .iter()
        .filter(|(k, _)| k != "digest")
        .cloned()
        .collect();
    let recomputed = bundle::fnv1a_hex(
        quorumforge::json::to_string(&quorumforge::json::Json::Obj(body)).as_bytes(),
    );
    assert_eq!(recomputed, stored);
}

#[test]
fn tampering_breaks_the_digest() {
    let (d, adj) = run("cache.qf", CACHE);
    let mut b = bundle::build(&d, &adj);
    // Flip one agent's weight; the stored digest should no longer match.
    if let Some(a) = b.deliberation.agents.get_mut("ada") {
        a.weight += 1.0;
    }
    assert!(
        !bundle::verify(&b),
        "mutating the body must invalidate the digest"
    );
}

#[test]
fn cache_sample_yields_expected_outcomes() {
    let (_d, adj) = run("cache.qf", CACHE);
    assert_eq!(adj.verdicts["c1"].outcome, Outcome::Consensus);
    assert!(adj.verdicts["c1"].affirmed);
    assert_eq!(adj.verdicts["c2"].outcome, Outcome::Contested);
    assert_eq!(adj.verdicts["c5"].outcome, Outcome::Unsupported);
    // The sample is designed to exercise all four outcomes.
    assert!(adj.tally.consensus >= 1);
    assert!(adj.tally.contested >= 1);
    assert!(adj.tally.split >= 1);
    assert!(adj.tally.unsupported >= 1);
}

#[test]
fn migration_sample_parses_and_adjudicates() {
    let (d, adj) = run("migration.json", MIGRATION);
    assert_eq!(d.id, "migration-strategy");
    assert_eq!(d.agents.len(), 5);
    assert_eq!(d.claims.len(), 6);
    assert_eq!(adj.tally.total(), 6);
    // m4 (independent shipping) has four supporters and no dissent.
    assert_eq!(adj.verdicts["m4"].outcome, Outcome::Consensus);
    assert!(adj.verdicts["m4"].affirmed);
    // m6 is only abstentions.
    assert_eq!(adj.verdicts["m6"].outcome, Outcome::Unsupported);
}

#[test]
fn normalization_collapses_hedged_variants() {
    let (d, _adj) = run("norm.qf", NORM);
    let n1 = &d.claims["n1"].normalized;
    let n2 = &d.claims["n2"].normalized;
    let n3 = &d.claims["n3"].normalized;
    assert_eq!(n1, n2);
    assert_eq!(n2, n3);
    assert_eq!(n1, "the p99 latency is under 200ms");
}

#[test]
fn normalize_text_expands_contractions() {
    assert_eq!(normalize_text("It isn't broken."), "it is not broken");
    assert_eq!(normalize_text("We can't ship."), "we cannot ship");
}

#[test]
fn normalize_text_strips_leading_hedges() {
    assert_eq!(normalize_text("I think that X holds"), "x holds");
    assert_eq!(normalize_text("Arguably, X holds"), "x holds");
    assert_eq!(normalize_text("Clearly X holds!"), "x holds");
}

#[test]
fn normalize_text_keeps_boundary_words() {
    // "likely" is a hedge, but "likelihood" must not be truncated.
    assert_eq!(
        normalize_text("Likelihood ratios matter"),
        "likelihood ratios matter"
    );
}

#[test]
fn clustering_groups_synonymous_claims() {
    let (d, _adj) = run("norm.qf", NORM);
    let clusters = normalize::cluster_by_normalized(&d);
    let group = clusters
        .get("the p99 latency is under 200ms")
        .expect("cluster exists");
    assert_eq!(group.len(), 3);
    assert_eq!(
        group,
        &vec!["n1".to_string(), "n2".to_string(), "n3".to_string()]
    );
}

#[test]
fn similarity_is_jaccard_over_tokens() {
    assert!((similarity("a b c", "a b c") - 1.0).abs() < 1e-9);
    assert!((similarity("a b", "c d") - 0.0).abs() < 1e-9);
    // {a,b,c} vs {a,b} -> intersection 2, union 3.
    assert!((similarity("a b c", "a b") - 2.0 / 3.0).abs() < 1e-9);
}

#[test]
fn end_to_end_run_matches_direct_pipeline() {
    let (_d, adj_direct) = run("cache.qf", CACHE);
    let (_d2, adj_lib) = quorumforge::run("cache.qf", CACHE, &Policy::default()).unwrap();
    assert_eq!(adj_direct.verdicts, adj_lib.verdicts);
    assert_eq!(adj_direct.cohesion, adj_lib.cohesion);
}
