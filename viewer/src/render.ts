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
  gray: "\x1b[90m",
};

const OUTCOME_META: Record<
  Outcome,
  { glyph: string; color: string; label: string }
> = {
  consensus: { glyph: "◆", color: ANSI.green, label: "CONSENSUS" },
  contested: { glyph: "▲", color: ANSI.red, label: "CONTESTED" },
  split: { glyph: "◈", color: ANSI.yellow, label: "SPLIT" },
  unsupported: { glyph: "○", color: ANSI.gray, label: "UNSUPPORTED" },
};

export interface ConsoleOptions {
  /** Disable ANSI colour codes (for piping to a file). */
  noColor?: boolean;
  /** Column width for the claim text column. */
  width?: number;
}

/** Render a report as a coloured, fixed-width console digest. */
export function renderConsole(report: Report, opts: ConsoleOptions = {}): string {
  const width = opts.width ?? 60;
  const paint = (code: string, text: string): string =>
    opts.noColor ? text : `${code}${text}${ANSI.reset}`;

  const lines: string[] = [];
  const rule = "─".repeat(74);
  const heavy = "━".repeat(74);

  lines.push(paint(ANSI.magenta, heavy));
  lines.push(
    paint(ANSI.bold, `QUORUMFORGE COUNCIL  ::  ${report.deliberation_id}`),
  );
  for (const l of wrap(report.question, 68)) {
    lines.push(paint(ANSI.dim, `  ${l}`));
  }
  lines.push(paint(ANSI.magenta, heavy));
