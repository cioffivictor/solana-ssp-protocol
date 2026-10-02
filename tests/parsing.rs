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
    let src = "\
delib | d1 | Q
agent | a | A | 1.0 | r
claim | c1 | t | text
pos   | ghost | c1 | support | 1.0 |
";
    let err = parse::parse_lines(src).unwrap_err();
    assert!(err.message.contains("unknown agent"));
}

#[test]
fn confidence_out_of_range_is_rejected() {
    let src = "\
delib | d1 | Q
agent | a | A | 1.0 | r
claim | c1 | t | text
pos   | a | c1 | support | 2.0 |
";
    let err = parse::parse_lines(src).unwrap_err();
    assert!(err.message.contains("outside [0,1]"));
}

#[test]
fn missing_delib_header_is_rejected() {
    let src = "agent | a | A | 1.0 | r\n";
    let err = parse::parse_lines(src).unwrap_err();
    assert!(err.message.contains("before the 'delib' header"));
}

#[test]
fn parses_json_format_equivalently() {
    let src = r#"
    {
      "id": "d1",
      "question": "Does it work?",
      "agents": [ { "id": "a", "name": "Alice", "weight": 1.5, "role": "dev" } ],
      "claims": [ { "id": "c1", "topic": "t", "text": "It works." } ],
      "positions": [
        { "agent": "a", "claim": "c1", "stance": "contradict", "confidence": 0.3,
          "note": "nope", "citations": [ { "source": "s", "locator": "l" } ] }
      ]
    }
    "#;
    let d = parse::parse_json(src).unwrap();
    assert_eq!(d.id, "d1");
    assert_eq!(d.agents["a"].weight, 1.5);
    assert_eq!(d.positions[0].stance, Stance::Contradict);
    assert_eq!(d.positions[0].citations.len(), 1);
}

#[test]
fn json_roundtrip_is_stable() {
    let src = r#"{"b":2,"a":[1,2,3],"nested":{"x":true,"y":null},"s":"hi\n"}"#;
    let parsed = json::parse(src).unwrap();
    let out1 = json::to_string(&parsed);
    let reparsed = json::parse(&out1).unwrap();
    let out2 = json::to_string(&reparsed);
    assert_eq!(out1, out2, "serialisation must be idempotent");
    assert_eq!(parsed, reparsed, "value must survive a round trip");
}

#[test]
fn json_preserves_object_key_order() {
    let src = r#"{"zebra":1,"apple":2,"mango":3}"#;
    let parsed = json::parse(src).unwrap();
    let out = json::to_string(&parsed);
    assert_eq!(out, src, "insertion order is preserved for determinism");
}

#[test]
fn json_handles_unicode_escapes() {
    let src = r#""caf\u00e9 \ud83d\ude00""#;
    let parsed = json::parse(src).unwrap();
    assert_eq!(parsed.as_str().unwrap(), "café 😀");
}

#[test]
fn json_rejects_trailing_garbage() {
    assert!(json::parse("{} oops").is_err());
    assert!(json::parse("[1,2,").is_err());
}

#[test]
fn integral_floats_serialize_without_decimal() {
    let v = Json::Num(42.0);
    assert_eq!(json::to_string(&v), "42");
    let v = Json::Num(1.5);
    assert_eq!(json::to_string(&v), "1.5");
}

#[test]
fn tiny_values_snap_and_round_trip() {
    // 1e-9 rounds to 0 on the six-decimal grid; the writer must be idempotent.
    let v = Json::Num(1e-9);
    let out1 = json::to_string(&v);
    let reparsed = json::parse(&out1).unwrap();
    let out2 = json::to_string(&reparsed);
    assert_eq!(out1, out2);
}
