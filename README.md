<!-- QuorumForge — a multi-agent evidence adjudication engine. -->

<div align="center">

# ✦ QuorumForge ✦

### *Hold the evidence to the light. Watch the verdict refract.*

<img src="docs/assets/council-prism.svg" alt="A beam of mixed evidence entering a prism and refracting into four verdict bands" width="640" />

**QuorumForge** takes the recorded deliberation of a council of agents — who
supported what, who pushed back, how sure they were, and what they cited — and
passes it through a prism of weighted reasoning. Out the far side come four
clean bands of light: **consensus**, **split**, **contested**, and
**unsupported**. Deterministic. Auditable. Re-derivable by hand.

</div>

---

## The one-paragraph pitch

A council argues. Somebody has to write down what the argument *concluded*.
QuorumForge is that scribe — and, more importantly, that *judge*. It is **not an
agent orchestrator**: it does not run agents, call models, or manage prompts. It
operates purely on the *record* of a deliberation after the debate is over. That
separation is the whole point. Because the engine only reads a static transcript
of positions, its judgements are reproducible: the same evidence file always
yields the same verdict, the same report bytes, and the same integrity digest.
You can commit a verdict to version control and diff it next week.

---

## Why a prism?

White light looks like a single, undifferentiated thing until a prism teases it
apart. A pile of agent opinions is the same: it *feels* like noise until you
separate it by direction and strength. QuorumForge's prism has exactly four
exit angles, and every claim leaves through exactly one of them.

| Band | Glyph | What it means |
|------|:-----:|---------------|
| **Consensus**   | ◆ | Strong agreement with little dissent. Affirmed or negated. |
| **Split**       | ◈ | A real lean, but below the consensus bar. Not yet settled. |
| **Contested**   | ▲ | Meaningful weight on *both* sides. The council is divided. |
| **Unsupported** | ○ | Too little decisive weight to conclude anything at all. |

The "unsupported" band is not a failure mode — it is a finding. Knowing that a
claim has *no* backing is often as valuable as knowing it is true.

---

## Anatomy of the light

QuorumForge is a small, sharp, mixed-language toolkit with no third-party
dependencies on either side of the fence.

```
quorumforge/
├── src/                      Rust core + CLI (std library only)
│   ├── lib.rs                public API and the end-to-end `run`
│   ├── model.rs              Agent / Claim / Position / Deliberation
│   ├── json.rs               a compact, idempotent JSON codec
│   ├── parse.rs              line-oriented (.qf) and JSON parsers
│   ├── normalize.rs          claim canonicalization + clustering
│   ├── adjudicate.rs         the weighted verdict engine
│   ├── bundle.rs             deterministic, digest-stamped bundles
│   ├── report.rs             text + JSON report renderers
│   └── bin/quorumforge.rs    the `quorumforge` command-line tool
├── tests/                    focused Rust integration tests
├── viewer/                   TypeScript council viewer (dependency-light)
│   └── src/                  report model, ANSI + HTML renderers, CLI, tests
├── samples/                  rich sample deliberations (.qf and .json)
├── docs/
│   ├── EVIDENCE.md           the normative input-format reference
│   └── assets/               the two animated SVGs on this page
├── Cargo.toml   Makefile   LICENSE   CHANGELOG.md   .gitignore
└── .github/workflows/ci.yml
```

Two languages, one contract: the Rust core emits a `quorumforge.report.v1` JSON
document, and the TypeScript viewer consumes exactly that schema. Neither side
pulls in a runtime dependency — the Rust crate is standard-library-only, and the
viewer's sole `devDependency` is the TypeScript compiler itself.

---

## Quick start

You need a Rust toolchain (1.74+) for the engine and Node.js (18+) plus npm for
the optional viewer. Nothing else.

```sh
# 1. Build and test the Rust engine.
cargo build --release
cargo test

# 2. Adjudicate a sample deliberation as a console report.
cargo run --release -- adjudicate samples/cache-coherence.qf
```

That last command prints something like:

```text
========================================================================
QUORUMFORGE VERDICT  ::  cache-coherence
Question: Is the distributed write-back cache coherent under concurrent
          writes?
========================================================================
Claims: 5   Consensus: 1   Contested: 1   Split: 2   Unsupported: 1
Council cohesion: 60.1%   Policy: consensus>=0.66, dissent<0.34
------------------------------------------------------------------------
[=] [c1] Writes to a single key are linearizable.
     consensus/affirmed topic=linearizability  polarity=+1.00  mass=4.33  ...
[!] [c2] Writes across multiple keys are linearizable.
     contested/negated topic=linearizability  polarity=-0.24  ...
     dissenting: ada, boro
...
```

---

