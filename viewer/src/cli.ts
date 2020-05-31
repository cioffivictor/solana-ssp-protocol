// qf-view — the QuorumForge council viewer CLI.
//
// Reads a `quorumforge.report.v1` JSON document (from a file or stdin) and
// renders it as a coloured console digest or a self-contained HTML page.
//
// Usage:
//   qf-view [options] <report.json | ->
//
// Options:
//   --html            Emit a standalone HTML document instead of console text.
//   --no-color        Disable ANSI colour in console output.
//   --width <n>       Claim-text column width for console output (default 60).
//   -o, --output <f>  Write to a file instead of stdout.
//   -h, --help        Show this help.

import { readFileSync, writeFileSync } from "node:fs";
import { parseReport } from "./report.js";
import { renderConsole, renderHtml } from "./render.js";

const HELP = `qf-view — QuorumForge council viewer

