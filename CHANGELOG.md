# Changelog

All notable changes to QuorumForge are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
to adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-08-31

The first release. QuorumForge arrives as a complete, dependency-free evidence
adjudication engine with a matching council viewer.

### Added

- **Rust core (std-only).**
  - Data model for agents, claims, positions, citations, and deliberations.
  - Two input parsers that converge on one model: a line-oriented `.qf` format
    (comments, escapes, `cite` attachment) and a JSON format.
  - A compact, idempotent JSON codec — parser and writer — with insertion-order
    preservation, unicode escape and surrogate-pair handling, and six-decimal
    number snapping for byte-stable output.
  - Lexical claim normalization: whitespace collapse, trailing-punctuation
    stripping, leading-hedge removal with word-boundary safety, and contraction
    expansion; plus normalized clustering and a Jaccard similarity helper.
  - A weighted adjudication engine producing four outcomes — consensus,
    contested, split, unsupported — with a tunable policy (consensus threshold,
    dissent ceiling, minimum mass) and a mass-weighted cohesion score.
  - Deterministic, digest-stamped evidence bundles (FNV-1a) with a `verify`
    round-trip guarantee.
  - Text and `quorumforge.report.v1` JSON report renderers.
- **CLI** (`quorumforge`) with `adjudicate`, `bundle`, `verify`, `inspect`, and
  `help` commands; stdin support; policy flags; and meaningful exit codes.
- **TypeScript council viewer** (`qf-view`), dependency-light (only `tsc` at
