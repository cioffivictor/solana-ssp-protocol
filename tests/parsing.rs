//! Tests for the line-oriented and JSON parsers and the JSON codec.

use quorumforge::json::{self, Json};
use quorumforge::model::Stance;
use quorumforge::parse;

#[test]
fn parses_minimal_line_format() {
    let src = "\
delib | d1 | Does it work?
agent | a | Alice | 1.0 | dev
claim | c1 | t | It works.
pos   | a | c1 | support | 0.9 | seems fine
";
    let d = parse::parse_lines(src).unwrap();
    assert_eq!(d.id, "d1");
    assert_eq!(d.question, "Does it work?");
    assert_eq!(d.agents.len(), 1);
    assert_eq!(d.claims.len(), 1);
    assert_eq!(d.positions.len(), 1);
    assert_eq!(d.positions[0].stance, Stance::Support);
    assert!((d.positions[0].confidence - 0.9).abs() < 1e-9);
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let src = "\
# a comment
delib | d1 | Q

# another
agent | a | A | 1.0 | r
claim | c1 | t | text
pos   | a | c1 | support | 1.0 |
";
    let d = parse::parse_lines(src).unwrap();
    assert_eq!(d.agents.len(), 1);
}

#[test]
fn cite_attaches_to_last_matching_position() {
    let src = "\
delib | d1 | Q
agent | a | A | 1.0 | r
claim | c1 | t | text
pos   | a | c1 | support | 1.0 | first
cite  | a | c1 | doc:one | page 3
cite  | a | c1 | doc:two | page 4
";
    let d = parse::parse_lines(src).unwrap();
    assert_eq!(d.positions[0].citations.len(), 2);
    assert_eq!(d.positions[0].citations[0].source, "doc:one");
    assert_eq!(d.positions[0].citations[1].locator, "page 4");
}

#[test]
fn escaped_pipe_is_literal() {
    let src = "\
delib | d1 | Q
agent | a | A | 1.0 | r
claim | c1 | t | a \\| b is a pipe
pos   | a | c1 | support | 1.0 |
";
    let d = parse::parse_lines(src).unwrap();
    assert_eq!(d.claims["c1"].text, "a | b is a pipe");
}

#[test]
fn unknown_stance_is_rejected() {
    let src = "\
delib | d1 | Q
agent | a | A | 1.0 | r
claim | c1 | t | text
pos   | a | c1 | maybe | 1.0 |
";
    let err = parse::parse_lines(src).unwrap_err();
    assert!(err.message.contains("unknown stance"));
}

#[test]
fn position_referencing_unknown_agent_is_rejected() {
