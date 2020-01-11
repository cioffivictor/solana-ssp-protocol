//! The QuorumForge data model.
//!
//! A *deliberation* is the top-level unit: a bounded question submitted to a
//! council of agents. Each [`Agent`] carries a credibility weight. Each
//! [`Claim`] is a normalized proposition about the question. Agents attach
//! [`Position`]s to claims — supporting, contradicting, or abstaining — each
//! with a confidence and optional [`Citation`]s.
//!
//! The engine never mutates a deliberation; it derives verdicts from it. That
//! separation is what makes the whole pipeline deterministic and testable.

use std::collections::BTreeMap;

/// The stance an agent takes toward a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
