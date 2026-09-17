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
            pos("b", "c1", Stance::Support, 1.0),
        ],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    assert_eq!(v.outcome, Outcome::Consensus);
    assert!(v.affirmed);
    assert!((v.polarity - 1.0).abs() < 1e-9);
    assert_eq!(v.supporters, 2);
    assert_eq!(v.dissenters, 0);
    assert!((v.dissent_ratio - 0.0).abs() < 1e-9);
}

#[test]
fn unanimous_contradiction_is_consensus_negated() {
    let d = build(
        vec![agent("a", 1.0), agent("b", 2.0)],
        vec![claim("c1")],
        vec![
            pos("a", "c1", Stance::Contradict, 1.0),
            pos("b", "c1", Stance::Contradict, 0.9),
        ],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    assert_eq!(v.outcome, Outcome::Consensus);
    assert!(!v.affirmed, "consensus that a claim is false is 'negated'");
    assert!(v.polarity < 0.0);
}

#[test]
fn near_even_split_is_contested() {
    let d = build(
        vec![agent("a", 1.0), agent("b", 1.0)],
        vec![claim("c1")],
        vec![
            pos("a", "c1", Stance::Support, 1.0),
            pos("b", "c1", Stance::Contradict, 1.0),
        ],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    assert_eq!(v.outcome, Outcome::Contested);
    assert!((v.dissent_ratio - 0.5).abs() < 1e-9);
    assert!((v.polarity - 0.0).abs() < 1e-9);
}

#[test]
fn moderate_lean_below_threshold_is_split() {
    // 60/40 support: polarity 0.2, dissent 0.4. Dissent 0.4 >= 0.34 ceiling
    // would make it contested, so tune to 70/30 -> polarity 0.4, dissent 0.3.
    let d = build(
        vec![agent("a", 0.7), agent("b", 0.3)],
        vec![claim("c1")],
        vec![
            pos("a", "c1", Stance::Support, 1.0),
            pos("b", "c1", Stance::Contradict, 1.0),
        ],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    assert_eq!(v.outcome, Outcome::Split);
    assert!(v.affirmed);
    assert!((v.polarity - 0.4).abs() < 1e-9);
    assert!((v.dissent_ratio - 0.3).abs() < 1e-9);
}

#[test]
fn only_abstentions_is_unsupported() {
    let d = build(
        vec![agent("a", 1.0), agent("b", 1.0)],
        vec![claim("c1")],
        vec![
            pos("a", "c1", Stance::Abstain, 0.5),
            pos("b", "c1", Stance::Abstain, 0.5),
        ],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    assert_eq!(v.outcome, Outcome::Unsupported);
    assert_eq!(v.abstentions, 2);
    assert_eq!(v.decisive_mass, 0.0);
}

#[test]
fn no_positions_at_all_is_unsupported() {
    let d = build(vec![agent("a", 1.0)], vec![claim("lonely")], vec![]);
    let v = verdict_for(&d, "lonely", &Policy::default());
    assert_eq!(v.outcome, Outcome::Unsupported);
}

#[test]
fn agent_weight_scales_influence() {
    // A single heavyweight supporter should outweigh two light contradictors.
    let d = build(
        vec![agent("heavy", 3.0), agent("l1", 1.0), agent("l2", 1.0)],
        vec![claim("c1")],
        vec![
            pos("heavy", "c1", Stance::Support, 1.0),
            pos("l1", "c1", Stance::Contradict, 1.0),
            pos("l2", "c1", Stance::Contradict, 0.0),
        ],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    // support 3.0, contradiction 1.0 -> polarity 0.5, dissent 0.25.
    assert!(v.affirmed);
    assert!((v.polarity - 0.5).abs() < 1e-9);
    assert_eq!(v.outcome, Outcome::Split);
}

#[test]
fn confidence_is_clamped_defensively() {
    // Confidence out of range would normally be rejected by the parser, but
    // the engine still clamps so a hand-built deliberation cannot explode.
    let d = build(
        vec![agent("a", 1.0)],
        vec![claim("c1")],
        vec![pos("a", "c1", Stance::Support, 5.0)],
    );
    let v = verdict_for(&d, "c1", &Policy::default());
    assert!((v.support_mass - 1.0).abs() < 1e-9, "5.0 clamps to 1.0");
}

#[test]
fn cohesion_is_mass_weighted() {
    // One unanimous heavy claim and one perfectly split light claim.
    let d = build(
        vec![agent("a", 4.0), agent("b", 1.0)],
        vec![claim("heavy"), claim("split")],
        vec![
            pos("a", "heavy", Stance::Support, 1.0),
            pos("b", "split", Stance::Support, 1.0),
            pos("a", "split", Stance::Contradict, 0.25),
        ],
    );
    let adj = adjudicate(&d, &Policy::default());
    // heavy: mass 4, polarity 1. split: support 1, contradict 1 -> mass 2,
    // polarity 0. Cohesion = (1*4 + 0*2) / (4+2) = 4/6 ~= 0.667.
    assert!((adj.cohesion - (4.0 / 6.0)).abs() < 1e-6);
}

#[test]
fn policy_thresholds_change_classification() {
    let d = build(
        vec![agent("a", 0.7), agent("b", 0.3)],
        vec![claim("c1")],
        vec![
            pos("a", "c1", Stance::Support, 1.0),
            pos("b", "c1", Stance::Contradict, 1.0),
        ],
    );
    // Default: 70/30 is a Split. Lower the consensus bar and raise dissent
    // tolerance so the same votes read as Consensus.
    let lenient = Policy {
        consensus_threshold: 0.3,
        dissent_ceiling: 0.5,
        minimum_mass: 0.0,
    };
    let v = verdict_for(&d, "c1", &lenient);
    assert_eq!(v.outcome, Outcome::Consensus);
}

#[test]
fn tally_counts_every_outcome_once() {
    let d = build(
        vec![agent("a", 1.0), agent("b", 1.0)],
        vec![claim("con"), claim("cont"), claim("unsup")],
        vec![
            pos("a", "con", Stance::Support, 1.0),
            pos("b", "con", Stance::Support, 1.0),
            pos("a", "cont", Stance::Support, 1.0),
            pos("b", "cont", Stance::Contradict, 1.0),
            pos("a", "unsup", Stance::Abstain, 0.5),
        ],
    );
    let adj = adjudicate(&d, &Policy::default());
    assert_eq!(adj.tally.consensus, 1);
    assert_eq!(adj.tally.contested, 1);
    assert_eq!(adj.tally.unsupported, 1);
    assert_eq!(adj.tally.total(), 3);
}

#[test]
fn policy_validation_rejects_out_of_range() {
    let bad = Policy {
        consensus_threshold: 1.5,
        ..Policy::default()
    };
    assert!(bad.validate().is_err());
    assert!(Policy::default().validate().is_ok());
}
