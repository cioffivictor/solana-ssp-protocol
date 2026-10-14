# Tuning the policy (sketch)

The three knobs trade off against each other, and the defaults are a starting point,
not an answer.

| Knob | Raising it | Lowering it |
|---|---|---|
| `consensus_threshold` | fewer claims reach consensus | more do, with thinner majorities |
| `dissent_ceiling` | consensus survives more disagreement | a single dissent can block it |
| `minimum_mass` | more claims land in unsupported | weak claims get a verdict |

## Open questions

- Should the report print the knobs it ran with?  (It does, in the bundle.)
- Is there a sensible way to suggest a policy from the data, or is that the caller's job?
