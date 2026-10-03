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
      topic: "misc",
      normalized: "the sky tastes sweet",
      outcome: "unsupported",
      affirmed: true,
      polarity: 0,
      decisive_mass: 0,
      support_mass: 0,
      contradiction_mass: 0,
      dissent_ratio: 0,
      supporters: 0,
      dissenters: 0,
      abstentions: 2,
      citations: 0,
      majority_agents: [],
      minority_agents: [],
    },
  ],
};

let passed = 0;
function test(name: string, fn: () => void): void {
  try {
    fn();
    passed++;
    process.stdout.write(`  ok  ${name}\n`);
  } catch (err) {
    process.stderr.write(`FAIL  ${name}\n      ${(err as Error).message}\n`);
    process.exitCode = 1;
  }
}

test("parseReport accepts a valid report", () => {
  const r = parseReport(JSON.parse(JSON.stringify(SAMPLE)));
  assert.equal(r.deliberation_id, "demo");
  assert.equal(r.verdicts.length, 3);
});

test("parseReport rejects a wrong schema", () => {
  assert.throws(() => parseReport({ schema: "nope" }), /unexpected schema/);
});

test("parseReport rejects a non-object", () => {
  assert.throws(() => parseReport(42), /must be a JSON object/);
});

test("console output contains outcome labels", () => {
  const out = renderConsole(SAMPLE, { noColor: true });
  assert.ok(out.includes("CONSENSUS"));
  assert.ok(out.includes("CONTESTED"));
  assert.ok(out.includes("UNSUPPORTED"));
});

test("console output lists dissenting agents", () => {
  const out = renderConsole(SAMPLE, { noColor: true });
  assert.ok(out.includes("dissent: a"), "c2 dissent lists agent a");
});

test("no-color output has no ANSI escape codes", () => {
  const out = renderConsole(SAMPLE, { noColor: true });
  // eslint-disable-next-line no-control-regex
  assert.ok(!/\x1b\[/.test(out), "should contain no escape sequences");
});

test("colored output does contain ANSI codes", () => {
  const out = renderConsole(SAMPLE, { noColor: false });
  // eslint-disable-next-line no-control-regex
  assert.ok(/\x1b\[/.test(out), "should contain escape sequences");
});

test("console output shows the roster with both agents", () => {
  const out = renderConsole(SAMPLE, { noColor: true });
  assert.ok(out.includes("Ada"));
  assert.ok(out.includes("Bo"));
});

test("html output is a self-contained document", () => {
  const html = renderHtml(SAMPLE);
  assert.ok(html.startsWith("<!DOCTYPE html>"));
  assert.ok(html.includes("<style>"), "styles are inlined");
  assert.ok(!/https?:\/\//.test(html), "no remote resources referenced");
  assert.ok(html.includes("The sky scatters blue light."));
});

test("html escapes angle brackets in claim text", () => {
  const injected: Report = JSON.parse(JSON.stringify(SAMPLE));
  injected.verdicts[0].text = "1 < 2 && 3 > 2";
  const html = renderHtml(injected);
  assert.ok(html.includes("1 &lt; 2 &amp;&amp; 3 &gt; 2"));
  assert.ok(!html.includes("1 < 2 &&"));
});

test("renderers are deterministic", () => {
  assert.equal(renderConsole(SAMPLE, {}), renderConsole(SAMPLE, {}));
  assert.equal(renderHtml(SAMPLE), renderHtml(SAMPLE));
});

process.stdout.write(`\n${passed} viewer test(s) passed\n`);
