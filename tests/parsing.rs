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
