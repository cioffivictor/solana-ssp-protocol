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

USAGE:
  qf-view [options] <report.json | ->

OPTIONS:
  --html            Emit a standalone HTML document instead of console text.
  --no-color        Disable ANSI colour in console output.
  --width <n>       Claim-text column width for console output (default 60).
  -o, --output <f>  Write to a file instead of stdout.
  -h, --help        Show this help.

EXAMPLES:
  quorumforge adjudicate --format json samples/cache-coherence.qf | qf-view -
  qf-view --html report.json -o council.html
`;

interface Options {
  html: boolean;
  noColor: boolean;
  width: number;
  output: string | null;
  input: string | null;
}

function parseArgs(argv: string[]): Options {
  const opts: Options = {
    html: false,
    noColor: false,
    width: 60,
    output: null,
    input: null,
  };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    switch (arg) {
      case "-h":
      case "--help":
        process.stdout.write(HELP);
        process.exit(0);
        break;
      case "--html":
        opts.html = true;
        break;
      case "--no-color":
        opts.noColor = true;
