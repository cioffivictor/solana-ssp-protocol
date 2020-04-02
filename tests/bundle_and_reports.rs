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
