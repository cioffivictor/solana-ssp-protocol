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
