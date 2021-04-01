// Focused tests for the viewer's renderers and report validation. Run with
// `npm test` (which compiles first) or `node dist/render.test.js`. Uses only
// the Node built-in `assert` module — no test framework dependency.

import assert from "node:assert/strict";
import { parseReport, type Report } from "./report.js";
import { renderConsole, renderHtml } from "./render.js";

const SAMPLE: Report = {
  schema: "quorumforge.report.v1",
  deliberation_id: "demo",
  question: "Is the sky blue at noon?",
  summary: {
    total_claims: 3,
    consensus: 1,
    contested: 1,
    split: 0,
    unsupported: 1,
    cohesion: 0.62,
  },
  policy: { consensus_threshold: 0.66, dissent_ceiling: 0.34, minimum_mass: 0 },
  agents: [
    { id: "a", name: "Ada", role: "optics", weight: 1.5, influence: 2.1 },
    { id: "b", name: "Bo", role: "skeptic", weight: 1.0, influence: 0.8 },
  ],
  verdicts: [
    {
      claim: "c1",
      text: "The sky scatters blue light.",
      topic: "optics",
      normalized: "the sky scatters blue light",
      outcome: "consensus",
      affirmed: true,
      polarity: 1.0,
      decisive_mass: 2.5,
      support_mass: 2.5,
      contradiction_mass: 0,
      dissent_ratio: 0,
      supporters: 2,
      dissenters: 0,
      abstentions: 0,
      citations: 1,
      majority_agents: ["a", "b"],
      minority_agents: [],
    },
    {
      claim: "c2",
      text: "The sky is green.",
      topic: "optics",
      normalized: "the sky is green",
      outcome: "contested",
      affirmed: false,
      polarity: -0.2,
      decisive_mass: 2.0,
      support_mass: 0.8,
      contradiction_mass: 1.2,
      dissent_ratio: 0.4,
      supporters: 1,
      dissenters: 1,
      abstentions: 0,
      citations: 0,
      majority_agents: ["b"],
      minority_agents: ["a"],
    },
    {
      claim: "c3",
      text: "The sky tastes sweet.",
