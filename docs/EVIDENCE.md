# The QuorumForge Evidence Format

QuorumForge reads a *deliberation*: the recorded output of a council of agents
arguing about a set of claims. This document is the normative reference for both
supported input encodings. They carry identical information and converge on the
same in-memory model, so you may author whichever suits your workflow.

- **Line-oriented (`.qf`)** — human-friendly, diff-friendly, comment-friendly.
- **JSON (`.json`)** — machine-friendly, easy to generate from other tools.

Format is chosen by file extension. Files ending in `.json` are parsed as JSON;
everything else is parsed as the line-oriented format. When reading from stdin,
pass `--json` to force JSON parsing.

---

## 1. The conceptual model

A deliberation contains four kinds of entity:

| Entity     | Meaning                                                              |
|------------|---------------------------------------------------------------------|
| Agent      | A council member with a **credibility weight** and optional role.   |
| Claim      | A proposition under adjudication, with raw and normalized text.     |
| Position   | One agent's **stance** on one claim, with a confidence and note.    |
| Citation   | A source reference attached to a position.                          |

A **stance** is one of `support`, `contradict`, or `abstain`. A **confidence**
is a number in `[0.0, 1.0]`. A **weight** is any non-negative number; `1.0` is
the neutral default.

The engine never invents positions. If no agent takes a decisive stance on a
claim, that claim is reported as `unsupported` — the absence of evidence is
