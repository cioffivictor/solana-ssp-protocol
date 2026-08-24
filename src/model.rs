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
    /// The id of the claim this position addresses.
    pub claim_id: String,
    /// Whether the agent supports, contradicts, or abstains.
    pub stance: Stance,
    /// The agent's self-reported confidence in `[0.0, 1.0]`.
    pub confidence: f64,
    /// Zero or more citations backing this position.
    pub citations: Vec<Citation>,
    /// Free-form rationale captured verbatim from the deliberation.
    pub note: String,
}

/// A council member with a credibility weight.
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    /// Stable identifier, unique within a deliberation.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Credibility weight in `[0.0, +inf)`. Higher means more influence.
    pub weight: f64,
    /// Optional role label, e.g. `domain-expert`, `skeptic`, `archivist`.
    pub role: String,
}

/// A proposition under adjudication.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    /// Stable identifier, unique within a deliberation.
    pub id: String,
    /// The raw claim text as authored.
    pub text: String,
    /// The normalized form used for grouping and comparison (filled in by the
    /// normalizer; equals `text` until then).
    pub normalized: String,
    /// Optional topic tag used to cluster related claims.
    pub topic: String,
}

/// A complete deliberation: the question, the council, the claims, and every
/// recorded position.
#[derive(Debug, Clone, PartialEq)]
pub struct Deliberation {
    /// Stable identifier for the deliberation.
    pub id: String,
    /// The question or motion the council is adjudicating.
    pub question: String,
    /// Council members, keyed by id for lookup.
    pub agents: BTreeMap<String, Agent>,
    /// Claims, keyed by id for lookup.
    pub claims: BTreeMap<String, Claim>,
    /// Every position taken during the deliberation, in authored order.
    pub positions: Vec<Position>,
}

impl Deliberation {
    /// Create an empty deliberation with the given id and question.
    pub fn new(id: impl Into<String>, question: impl Into<String>) -> Self {
        Deliberation {
            id: id.into(),
            question: question.into(),
            agents: BTreeMap::new(),
            claims: BTreeMap::new(),
            positions: Vec::new(),
        }
    }

    /// All positions that address a given claim.
    pub fn positions_for<'a>(&'a self, claim_id: &'a str) -> impl Iterator<Item = &'a Position> {
        self.positions
            .iter()
            .filter(move |p| p.claim_id == claim_id)
    }

    /// The weight assigned to an agent, defaulting to `1.0` when unknown.
    pub fn agent_weight(&self, agent_id: &str) -> f64 {
        self.agents.get(agent_id).map(|a| a.weight).unwrap_or(1.0)
    }
}
