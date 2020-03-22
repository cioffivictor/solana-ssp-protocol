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
        claim_id: claim.into(),
        stance,
        confidence,
        citations: Vec::new(),
        note: String::new(),
    }
}

fn build(agents: Vec<Agent>, claims: Vec<Claim>, positions: Vec<Position>) -> Deliberation {
    let mut d = Deliberation::new("t", "question");
    for a in agents {
        d.agents.insert(a.id.clone(), a);
    }
    for c in claims {
        d.claims.insert(c.id.clone(), c);
    }
    d.positions = positions;
    d
}

#[test]
fn unanimous_support_is_consensus_affirmed() {
    let d = build(
        vec![agent("a", 1.0), agent("b", 1.0)],
        vec![claim("c1")],
        vec![
            pos("a", "c1", Stance::Support, 1.0),
