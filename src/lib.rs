//! # QuorumForge
//!
//! QuorumForge is a **multi-agent evidence adjudication engine**. It ingests a
//! recorded deliberation — a council of weighted agents taking positions on a
//! set of claims, with citations and confidences — and turns it into a
//! deterministic verdict: which claims reached consensus, which are contested,
//! which merely lean one way, and which lack the evidence to conclude anything.
//!
//! It is *not* an agent orchestrator. QuorumForge does not run agents, call
//! models, or manage prompts. It operates purely on the *record* of a
//! deliberation after the fact, which is what makes its judgements auditable
//! and reproducible.
//!
//! ## Pipeline
//!
//! ```text
//!   evidence file            Deliberation           Adjudication
//!  (.qf or .json)  --parse-->  (model)   --adjudicate-->  (verdicts)
//!                                 |                            |
//!                             normalize                    report /
//!                              claims                       bundle
//! ```
//!
//! ## Example
//!
//! ```
//! use quorumforge::{parse, normalize, adjudicate};
//!
//! let src = "\
//! delib | d1 | Is the cache coherent under concurrent writes?
//! agent | a | Ada | 1.5 | systems
//! agent | b | Boro | 1.0 | skeptic
//! claim | c1 | cache | Writes are linearizable.
//! pos   | a  | c1 | support     | 0.9 | verified with a model checker
//! pos   | b  | c1 | contradict  | 0.4 | one race remains under retry
//! ";
//! let mut delib = parse::parse_lines(src).unwrap();
//! normalize::normalize_deliberation(&mut delib);
//! let adj = adjudicate::adjudicate(&delib, &adjudicate::Policy::default());
//! let verdict = &adj.verdicts["c1"];
//! assert!(verdict.support_mass > verdict.contradiction_mass);
//! ```

pub mod adjudicate;
pub mod bundle;
pub mod json;
pub mod model;
pub mod normalize;
pub mod parse;
pub mod report;
