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
- `stance` accepts synonyms: `support`/`supports`/`for`/`+`,
  `contradict`/`contradicts`/`refute`/`against`/`-`, `abstain`/`neutral`/`0`.

### 2.3 Worked example

```qf
# A three-agent deliberation with one clear consensus claim.
delib | demo | Is the release ready to ship?

agent | ana  | Ana Ito     | 1.4 | release-manager
agent | ben  | Ben Osei    | 1.0 | qa
agent | cse  | Cse Varga   | 0.9 | support

claim | r1 | quality | All P0 bugs are resolved.
claim | r2 | risk    | The rollback path has been rehearsed.

pos  | ana | r1 | support    | 0.9 | burn-down chart is at zero
cite | ana | r1 | tracker:P0 | filter status=closed returns 0 rows
pos  | ben | r1 | support    | 0.8 | verified the last three fixes
pos  | cse | r1 | abstain    | 0.5 | no visibility into P0 triage

pos  | ana | r2 | support    | 0.7 | dry run last Tuesday
pos  | ben | r2 | contradict | 0.6 | the dry run skipped the data migration
```

---

## 3. JSON format (`.json`)

A single object with these top-level keys:

| Key         | Type    | Required | Notes                                    |
|-------------|---------|----------|------------------------------------------|
| `id`        | string  | no       | Defaults to `"unnamed"`.                 |
| `question`  | string  | no       | Defaults to empty.                       |
| `agents`    | array   | no       | Array of agent objects.                  |
| `claims`    | array   | no       | Array of claim objects.                  |
| `positions` | array   | no       | Array of position objects.               |

### 3.1 Object shapes

```jsonc
// agent
{ "id": "ana", "name": "Ana Ito", "weight": 1.4, "role": "release-manager" }

// claim
{ "id": "r1", "topic": "quality", "text": "All P0 bugs are resolved." }

// position
{
  "agent": "ana",
  "claim": "r1",
  "stance": "support",          // support | contradict | abstain (+ synonyms)
  "confidence": 0.9,            // [0.0, 1.0]
  "note": "burn-down at zero",
  "citations": [
    { "source": "tracker:P0", "locator": "filter status=closed -> 0 rows" }
  ]
}
```

Field defaults match the line format: `weight` and `confidence` default to
`1.0`; `role`, `note`, `topic`, `source`, and `locator` default to empty;
`citations` defaults to an empty list.

### 3.2 Equivalent example

```json
{
  "id": "demo",
  "question": "Is the release ready to ship?",
  "agents": [
    { "id": "ana", "name": "Ana Ito", "weight": 1.4, "role": "release-manager" },
    { "id": "ben", "name": "Ben Osei", "weight": 1.0, "role": "qa" }
  ],
  "claims": [
    { "id": "r1", "topic": "quality", "text": "All P0 bugs are resolved." }
  ],
  "positions": [
    { "agent": "ana", "claim": "r1", "stance": "support", "confidence": 0.9,
      "note": "burn-down at zero",
      "citations": [ { "source": "tracker:P0", "locator": "0 open rows" } ] },
