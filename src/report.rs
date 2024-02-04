//! Human- and machine-readable reports.
//!
//! Two renderers share one source of truth (the [`Adjudication`]):
//!
//! * [`render_text`] produces a fixed-width console report with a headline,
//!   a per-claim table, and a dissent roster.
//! * [`render_json`] produces the structured report the TypeScript council
//!   viewer consumes. The JSON report is a superset of the bundle summary,
//!   enriched with the per-agent influence breakdown.
//!
//! Both renderers are deterministic and free of timestamps or environment
//! details, so their output can be committed as golden files.

use crate::adjudicate::{Adjudication, Outcome, Verdict};
use crate::json::{self, Json};
use crate::model::Deliberation;
use std::collections::BTreeMap;

/// Render a console-friendly text report.
pub fn render_text(delib: &Deliberation, adj: &Adjudication) -> String {
    let mut out = String::new();
    let bar = "=".repeat(72);
    let thin = "-".repeat(72);

    out.push_str(&bar);
    out.push('\n');
    out.push_str(&format!(
        "QUORUMFORGE VERDICT  ::  {}\n",
        adj.deliberation_id
    ));
    out.push_str(&format!(
        "Question: {}\n",
        wrap(&adj.question, 62, "          ")
    ));
    out.push_str(&bar);
    out.push('\n');

    // Headline tally.
    out.push_str(&format!(
        "Claims: {}   Consensus: {}   Contested: {}   Split: {}   Unsupported: {}\n",
        adj.tally.total(),
        adj.tally.consensus,
        adj.tally.contested,
        adj.tally.split,
        adj.tally.unsupported,
    ));
    out.push_str(&format!(
        "Council cohesion: {:.1}%   Policy: consensus>={:.2}, dissent<{:.2}\n",
        adj.cohesion * 100.0,
        adj.policy.consensus_threshold,
        adj.policy.dissent_ceiling,
    ));
    out.push_str(&thin);
    out.push('\n');

    // Per-claim rows.
    for verdict in adj.verdicts.values() {
        let claim = delib.claims.get(&verdict.claim_id);
        let text = claim.map(|c| c.text.as_str()).unwrap_or("(unknown claim)");
        let topic = claim.map(|c| c.topic.as_str()).unwrap_or("");
        let glyph = outcome_glyph(verdict.outcome);
        let direction = if verdict.outcome == Outcome::Unsupported {
            "n/a".to_string()
        } else if verdict.affirmed {
            "affirmed".to_string()
        } else {
            "negated".to_string()
        };
        out.push_str(&format!(
            "{} [{}] {}\n",
            glyph,
            verdict.claim_id,
            wrap(text, 58, "         "),
        ));
        let topic_label = if topic.is_empty() {
            String::new()
        } else {
            format!("topic={}  ", topic)
        };
        out.push_str(&format!(
            "     {:<11} {}polarity={:+.2}  mass={:.2}  dissent={:.0}%  cites={}\n",
            format!("{}/{}", verdict.outcome.as_token(), direction),
            topic_label,
            verdict.polarity,
            verdict.decisive_mass,
            verdict.dissent_ratio * 100.0,
            verdict.citation_count,
        ));
        if !verdict.minority_agents.is_empty() {
            out.push_str(&format!(
                "     dissenting: {}\n",
                verdict.minority_agents.join(", ")
            ));
        }
    }

    out.push_str(&thin);
    out.push('\n');

    // Per-agent influence roster.
    out.push_str("Agent influence (weighted decisive votes cast):\n");
    let influence = agent_influence(delib, adj);
    for (agent_id, score) in &influence {
        let name = delib
            .agents
            .get(agent_id)
            .map(|a| a.name.as_str())
            .unwrap_or(agent_id.as_str());
        out.push_str(&format!("  {:<16} {:>7.2}   {}\n", agent_id, score, name));
    }
    out.push_str(&bar);
    out.push('\n');
    out
}

fn outcome_glyph(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Consensus => "[=]",
        Outcome::Contested => "[!]",
        Outcome::Split => "[~]",
        Outcome::Unsupported => "[?]",
    }
}

/// Sum of weighted decisive votes each agent cast, in id order. Used by both
/// the text roster and the JSON report so they never disagree.
fn agent_influence(delib: &Deliberation, adj: &Adjudication) -> Vec<(String, f64)> {
    let mut scores: BTreeMap<String, f64> = BTreeMap::new();
    for agent_id in delib.agents.keys() {
        scores.insert(agent_id.clone(), 0.0);
    }
    for pos in &delib.positions {
        if pos.stance == crate::model::Stance::Abstain {
            continue;
        }
        let weight = delib.agent_weight(&pos.agent_id);
        let conf = pos.confidence.clamp(0.0, 1.0);
        *scores.entry(pos.agent_id.clone()).or_insert(0.0) += weight * conf;
    }
    // Suppress unused warning for adj; kept in signature for symmetry/testability.
    let _ = adj;
    scores.into_iter().collect()
}

/// Wrap `text` to `width` columns, indenting continuation lines with `indent`.
fn wrap(text: &str, width: usize, indent: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() > width {
            lines.push(current.clone());
            current.clear();
            current.push_str(word);
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join(&format!("\n{}", indent))
}

