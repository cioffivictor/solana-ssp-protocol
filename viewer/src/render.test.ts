// Focused tests for the viewer's renderers and report validation. Run with
// `npm test` (which compiles first) or `node dist/render.test.js`. Uses only
// the Node built-in `assert` module — no test framework dependency.

import assert from "node:assert/strict";
import { parseReport, type Report } from "./report.js";
import { renderConsole, renderHtml } from "./render.js";

const SAMPLE: Report = {
