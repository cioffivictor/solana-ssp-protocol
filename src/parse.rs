//! Parsers for the two documented QuorumForge evidence formats.
//!
//! ## Line-oriented format (`.qf`)
//!
//! A human-friendly, diff-friendly format. One record per line; blank lines and
//! lines beginning with `#` are ignored. Every record starts with a directive
//! keyword followed by pipe-delimited fields. The grammar is:
//!
//! ```text
//! delib   | <id> | <question>
//! agent   | <id> | <name> | <weight> | <role>
//! claim   | <id> | <topic> | <text>
//! pos     | <agent_id> | <claim_id> | <stance> | <confidence> | <note>
//! cite    | <agent_id> | <claim_id> | <source> | <locator>
//! ```
//!
//! `cite` attaches to the most recently declared position for that
//! (agent, claim) pair. Fields are trimmed of surrounding whitespace. A literal
//! pipe inside a field can be escaped as `\|`.
//!
//! ## JSON format (`.json`)
//!
//! The same information as a single JSON object. See `docs/EVIDENCE.md` for the
//! full schema. Both parsers converge on the same [`Deliberation`] value.

use crate::json::{self, Json};
use crate::model::{Agent, Citation, Claim, Deliberation, Position, Stance};

/// An error encountered while parsing evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "parse error: {}", self.message)
        } else {
            write!(f, "parse error on line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// Detect the format from a file extension and dispatch to the right parser.
pub fn parse_auto(path: &str, contents: &str) -> Result<Deliberation, ParseError> {
    let lower = path.to_lowercase();
    if lower.ends_with(".json") {
        parse_json(contents)
    } else {
        parse_lines(contents)
    }
}

/// Split a line into pipe-delimited fields, honouring `\|` escapes.
fn split_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if chars.peek() == Some(&'|') => {
                chars.next();
                current.push('|');
            }
            '|' => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Parse the line-oriented `.qf` format.
pub fn parse_lines(contents: &str) -> Result<Deliberation, ParseError> {
    let mut delib: Option<Deliberation> = None;

    for (idx, raw) in contents.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = split_fields(line);
        let directive = fields[0].to_ascii_lowercase();

        macro_rules! need {
            ($n:expr) => {
                if fields.len() < $n {
                    return Err(ParseError {
                        line: line_no,
                        message: format!(
                            "'{}' needs {} fields, found {}",
                            directive,
                            $n - 1,
                            fields.len() - 1
                        ),
                    });
                }
            };
        }

        match directive.as_str() {
            "delib" => {
                need!(3);
                if delib.is_some() {
                    return Err(ParseError {
                        line: line_no,
                        message: "only one 'delib' record is allowed per file".into(),
                    });
                }
                delib = Some(Deliberation::new(fields[1].clone(), fields[2].clone()));
            }
            "agent" => {
                need!(3);
                let d = require_delib(&mut delib, line_no)?;
                let weight = parse_f64(fields.get(3), 1.0, line_no, "weight")?;
                let role = fields.get(4).cloned().unwrap_or_default();
                let id = fields[1].clone();
                d.agents.insert(
                    id.clone(),
                    Agent {
                        id,
                        name: fields[2].clone(),
                        weight,
                        role,
                    },
                );
            }
            "claim" => {
                need!(4);
                let d = require_delib(&mut delib, line_no)?;
                let id = fields[1].clone();
                d.claims.insert(
                    id.clone(),
                    Claim {
                        id,
                        topic: fields[2].clone(),
                        text: fields[3].clone(),
                        normalized: String::new(),
                    },
                );
            }
            "pos" => {
                need!(5);
                let d = require_delib(&mut delib, line_no)?;
                let stance = Stance::parse(&fields[3]).ok_or_else(|| ParseError {
                    line: line_no,
                    message: format!("unknown stance '{}'", fields[3]),
                })?;
                let confidence = parse_f64(fields.get(4), 1.0, line_no, "confidence")?;
                let note = fields.get(5).cloned().unwrap_or_default();
                d.positions.push(Position {
                    agent_id: fields[1].clone(),
                    claim_id: fields[2].clone(),
                    stance,
                    confidence,
                    citations: Vec::new(),
                    note,
                });
            }
            "cite" => {
                need!(5);
                let d = require_delib(&mut delib, line_no)?;
                let agent_id = &fields[1];
                let claim_id = &fields[2];
                let citation = Citation::new(fields[3].clone(), fields[4].clone());
                // Attach to the most recent matching position.
                let target = d
                    .positions
                    .iter_mut()
                    .rev()
                    .find(|p| &p.agent_id == agent_id && &p.claim_id == claim_id);
                match target {
                    Some(pos) => pos.citations.push(citation),
                    None => {
                        return Err(ParseError {
                            line: line_no,
                            message: format!(
                                "'cite' references position ({}, {}) that has not been declared",
                                agent_id, claim_id
                            ),
                        })
                    }
                }
            }
            other => {
                return Err(ParseError {
                    line: line_no,
                    message: format!("unknown directive '{}'", other),
                });
            }
        }
    }

    let delib = delib.ok_or_else(|| ParseError {
        line: 0,
        message: "file contained no 'delib' record".into(),
    })?;
    validate(&delib)?;
    Ok(delib)
}

fn require_delib(
    delib: &mut Option<Deliberation>,
    line_no: usize,
) -> Result<&mut Deliberation, ParseError> {
    delib.as_mut().ok_or_else(|| ParseError {
        line: line_no,
        message: "record appeared before the 'delib' header".into(),
    })
}

fn parse_f64(
    field: Option<&String>,
    default: f64,
    line_no: usize,
    what: &str,
) -> Result<f64, ParseError> {
    match field {
        None => Ok(default),
        Some(s) if s.is_empty() => Ok(default),
        Some(s) => s.parse::<f64>().map_err(|_| ParseError {
            line: line_no,
            message: format!("invalid {} value '{}'", what, s),
        }),
    }
}

/// Parse the JSON evidence format.
pub fn parse_json(contents: &str) -> Result<Deliberation, ParseError> {
    let root = json::parse(contents).map_err(|e| ParseError {
        line: 0,
        message: e.message,
    })?;

    let id = str_field(&root, "id").unwrap_or_else(|| "unnamed".to_string());
    let question = str_field(&root, "question").unwrap_or_default();
    let mut delib = Deliberation::new(id, question);

    if let Some(agents) = root.get("agents").and_then(Json::as_array) {
        for a in agents {
            let id = str_field(a, "id").ok_or_else(|| field_err("agent.id"))?;
