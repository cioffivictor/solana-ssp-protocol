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
