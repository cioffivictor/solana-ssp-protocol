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

