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
