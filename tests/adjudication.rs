//! Tests for the verdict classifier and the weighted scoring model.

use quorumforge::adjudicate::{adjudicate, verdict_for, Outcome, Policy};
use quorumforge::model::{Agent, Claim, Deliberation, Position, Stance};

fn agent(id: &str, weight: f64) -> Agent {
    Agent {
        id: id.into(),
        name: id.into(),
        weight,
        role: String::new(),
    }
}

fn claim(id: &str) -> Claim {
    Claim {
        id: id.into(),
        text: format!("claim {}", id),
        normalized: format!("claim {}", id),
        topic: String::new(),
    }
}

fn pos(agent: &str, claim: &str, stance: Stance, confidence: f64) -> Position {
    Position {
        agent_id: agent.into(),
