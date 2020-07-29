// Renderers for QuorumForge reports. Two targets share one report model:
//   * renderConsole  -> ANSI-coloured, fixed-width council digest for a terminal
//   * renderHtml      -> a single self-contained HTML page (no remote assets)
//
// Both are pure functions of the report, so their output is deterministic and
// testable.

import type { Outcome, Report, VerdictView } from "./report.js";

const ANSI = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  dim: "\x1b[2m",
  green: "\x1b[32m",
  red: "\x1b[31m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  magenta: "\x1b[35m",
