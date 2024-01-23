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
pub enum Stance {
    /// The agent asserts the claim is true.
    Support,
    /// The agent asserts the claim is false.
    Contradict,
    /// The agent declines to take a side (recorded, but carries no weight).
    Abstain,
}

impl Stance {
    /// Parse a stance from its canonical lowercase token.
    pub fn parse(token: &str) -> Option<Stance> {
        match token.trim().to_ascii_lowercase().as_str() {
            "support" | "supports" | "+" | "for" => Some(Stance::Support),
            "contradict" | "contradicts" | "against" | "-" | "refute" | "refutes" => {
                Some(Stance::Contradict)
            }
            "abstain" | "abstains" | "neutral" | "0" => Some(Stance::Abstain),
            _ => None,
        }
    }

    /// The canonical lowercase token for this stance.
    pub fn as_token(self) -> &'static str {
        match self {
            Stance::Support => "support",
            Stance::Contradict => "contradict",
            Stance::Abstain => "abstain",
        }
    }

    /// The signed direction a stance contributes to a claim's weighted score.
    pub fn sign(self) -> f64 {
        match self {
            Stance::Support => 1.0,
            Stance::Contradict => -1.0,
            Stance::Abstain => 0.0,
        }
    }
}

/// A source reference attached to a position.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    /// A short, stable identifier for the source (e.g. `doc:rfc-8949`).
    pub source: String,
    /// A human-readable locator: page, section, URL fragment, or quote.
    pub locator: String,
}

impl Citation {
    pub fn new(source: impl Into<String>, locator: impl Into<String>) -> Self {
        Citation {
            source: source.into(),
            locator: locator.into(),
        }
    }
}

/// A single agent's stance on a single claim.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// The id of the agent that holds this position.
    pub agent_id: String,
