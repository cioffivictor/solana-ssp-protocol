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
        break;
      case "--width": {
        const v = argv[++i];
        const n = Number(v);
        if (!Number.isFinite(n) || n <= 0) {
          fail(`--width expects a positive number, got '${v}'`);
        }
        opts.width = Math.floor(n);
        break;
      }
      case "-o":
      case "--output":
        opts.output = argv[++i] ?? null;
        if (opts.output === null) fail("--output requires a path");
        break;
      default:
        if (arg.startsWith("--")) {
          fail(`unknown option '${arg}'`);
        } else if (opts.input === null) {
          opts.input = arg;
        } else {
          fail(`unexpected extra argument '${arg}'`);
        }
    }
  }
  return opts;
}

function fail(message: string): never {
  process.stderr.write(`qf-view: ${message}\n`);
  process.exit(2);
}

function readStdin(): Promise<string> {
  return new Promise((resolve) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      data += typeof chunk === "string" ? chunk : String(chunk);
    });
    process.stdin.on("end", () => resolve(data));
  });
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  if (argv.length === 0) {
    process.stdout.write(HELP);
    process.exit(2);
  }
  const opts = parseArgs(argv);
  if (opts.input === null) {
    fail("no report file given (use '-' for stdin)");
  }

  let raw: string;
  if (opts.input === "-") {
    raw = await readStdin();
  } else {
    try {
      raw = readFileSync(opts.input, "utf8");
    } catch (err) {
      fail(`cannot read '${opts.input}': ${(err as Error).message}`);
    }
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    fail(`input is not valid JSON: ${(err as Error).message}`);
  }

  let report;
  try {
    report = parseReport(parsed);
  } catch (err) {
    fail((err as Error).message);
  }

  const rendered = opts.html
    ? renderHtml(report)
    : renderConsole(report, {
        noColor: opts.noColor || opts.output !== null,
        width: opts.width,
      });

  if (opts.output) {
    writeFileSync(opts.output, rendered);
  } else {
    process.stdout.write(rendered);
  }
}

main().catch((err) => {
  process.stderr.write(`qf-view: ${(err as Error).message}\n`);
  process.exit(1);
});
