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
itself a first-class result.

---

## 2. Line-oriented format (`.qf`)

### 2.1 Lexical rules

- One **record** per line.
- Blank lines are ignored.
- Lines whose first non-space character is `#` are comments and are ignored.
- A record is a **directive keyword** followed by pipe-delimited (`|`) fields.
- Every field is trimmed of surrounding whitespace.
- A literal pipe inside a field is written `\|`.

### 2.2 Directives

```text
delib | <id> | <question>
agent | <id> | <name> | <weight> | <role>
claim | <id> | <topic> | <text>
pos   | <agent_id> | <claim_id> | <stance> | <confidence> | <note>
cite  | <agent_id> | <claim_id> | <source> | <locator>
```

Rules:

- Exactly **one** `delib` record is required, and it must appear before any
  `agent`, `claim`, `pos`, or `cite` record.
- `weight` defaults to `1.0` if omitted or empty; `role` defaults to empty.
- `confidence` defaults to `1.0` if omitted or empty; `note` defaults to empty.
- A `cite` attaches to the **most recently declared** `pos` for the same
  `(agent_id, claim_id)` pair. A `cite` with no matching position is an error.
