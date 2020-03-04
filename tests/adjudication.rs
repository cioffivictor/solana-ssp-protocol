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
