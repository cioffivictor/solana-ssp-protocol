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

  const s = report.summary;
  const cohesionBar = bar(s.cohesion, 20);
  lines.push(
    `claims ${s.total_claims}   ` +
      paint(ANSI.green, `consensus ${s.consensus}`) +
      "   " +
      paint(ANSI.red, `contested ${s.contested}`) +
      "   " +
      paint(ANSI.yellow, `split ${s.split}`) +
      "   " +
      paint(ANSI.gray, `unsupported ${s.unsupported}`),
  );
  lines.push(
    `cohesion ${paint(ANSI.cyan, cohesionBar)} ${(s.cohesion * 100).toFixed(1)}%`,
  );
  lines.push(paint(ANSI.gray, rule));

  for (const v of report.verdicts) {
    lines.push(...renderVerdictConsole(v, width, paint));
  }

  lines.push(paint(ANSI.gray, rule));
  lines.push(paint(ANSI.bold, "council roster (weighted influence)"));
  const maxInfluence = Math.max(1e-9, ...report.agents.map((a) => a.influence));
  for (const a of report.agents) {
    const b = bar(a.influence / maxInfluence, 16);
    lines.push(
      `  ${pad(a.id, 14)} ${paint(ANSI.cyan, b)} ${a.influence
        .toFixed(2)
        .padStart(6)}  ${paint(ANSI.dim, `${a.name} · ${a.role || "—"}`)}`,
    );
  }
  lines.push(paint(ANSI.magenta, heavy));
  return lines.join("\n") + "\n";
}

