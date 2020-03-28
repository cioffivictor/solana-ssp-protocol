//! Tests for deterministic bundles, normalization, and end-to-end runs against
//! the checked-in sample deliberations.

use quorumforge::adjudicate::{adjudicate, Outcome, Policy};
use quorumforge::normalize::{normalize_text, similarity};
use quorumforge::{bundle, normalize, parse};
