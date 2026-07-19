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

function renderVerdictConsole(
  v: VerdictView,
  width: number,
  paint: (code: string, t: string) => string,
): string[] {
  const meta = OUTCOME_META[v.outcome];
  const out: string[] = [];
  const direction =
    v.outcome === "unsupported" ? "" : v.affirmed ? " affirmed" : " negated";
  const head = `${meta.glyph} [${v.claim}] `;
  const wrapped = wrap(v.text, width);
  out.push(paint(meta.color, `${head}${wrapped[0] ?? ""}`));
  for (const cont of wrapped.slice(1)) {
    out.push(paint(meta.color, `${" ".repeat(head.length)}${cont}`));
  }
  const polarity = (v.polarity >= 0 ? "+" : "") + v.polarity.toFixed(2);
  out.push(
    paint(
      ANSI.dim,
      `    ${meta.label}${direction}  ` +
        `polarity ${polarity}  mass ${v.decisive_mass.toFixed(2)}  ` +
        `dissent ${(v.dissent_ratio * 100).toFixed(0)}%  cites ${v.citations}`,
    ),
  );
  if (v.minority_agents.length > 0) {
    out.push(paint(ANSI.red, `    dissent: ${v.minority_agents.join(", ")}`));
  }
  return out;
}

/** Render a small unicode meter for a value in [0, 1]. */
function bar(fraction: number, cells: number): string {
  const clamped = Math.max(0, Math.min(1, fraction));
  const filled = Math.round(clamped * cells);
  return "█".repeat(filled) + "░".repeat(cells - filled);
}

function pad(text: string, len: number): string {
  return text.length >= len ? text : text + " ".repeat(len - text.length);
}

function wrap(text: string, width: number): string[] {
  const words = text.split(/\s+/).filter(Boolean);
  const lines: string[] = [];
  let current = "";
  for (const word of words) {
    if (current === "") {
      current = word;
    } else if (current.length + 1 + word.length > width) {
      lines.push(current);
      current = word;
    } else {
      current += " " + word;
    }
  }
  if (current) lines.push(current);
  return lines.length ? lines : [""];
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Render a report as a single self-contained HTML document. All styling is
 * inline; there are no remote fonts, scripts, or images. The result can be
 * opened directly in a browser or committed as an artifact.
 */
export function renderHtml(report: Report): string {
  const s = report.summary;
  const verdictCards = report.verdicts
    .map((v) => {
      const meta = OUTCOME_META[v.outcome];
      const hue =
        v.outcome === "consensus"
          ? 145
          : v.outcome === "contested"
            ? 5
            : v.outcome === "split"
              ? 45
              : 220;
      const dir =
        v.outcome === "unsupported" ? "" : v.affirmed ? "affirmed" : "negated";
      const dissent =
        v.minority_agents.length > 0
          ? `<div class="dissent">dissent: ${escapeHtml(
              v.minority_agents.join(", "),
            )}</div>`
          : "";
      return `      <article class="card" style="--hue:${hue}">
        <header><span class="pill">${meta.label}${dir ? " · " + dir : ""}</span>
          <code>${escapeHtml(v.claim)}</code></header>
        <p class="claim">${escapeHtml(v.text)}</p>
        <div class="meter" title="polarity ${v.polarity.toFixed(2)}">
          <span style="width:${((v.polarity + 1) / 2) * 100}%"></span>
        </div>
        <dl>
          <div><dt>polarity</dt><dd>${v.polarity.toFixed(2)}</dd></div>
          <div><dt>mass</dt><dd>${v.decisive_mass.toFixed(2)}</dd></div>
          <div><dt>dissent</dt><dd>${(v.dissent_ratio * 100).toFixed(0)}%</dd></div>
          <div><dt>cites</dt><dd>${v.citations}</dd></div>
        </dl>${dissent}
      </article>`;
    })
    .join("\n");

  const roster = report.agents
    .map(
      (a) =>
        `        <li><b>${escapeHtml(a.id)}</b> <span>${escapeHtml(
          a.name,
        )} · ${escapeHtml(a.role || "—")}</span>
          <div class="bar"><span style="width:${Math.min(
            100,
            (a.influence / Math.max(1e-9, ...report.agents.map((x) => x.influence))) *
              100,
          )}%"></span></div>
          <em>${a.influence.toFixed(2)}</em></li>`,
    )
    .join("\n");

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>QuorumForge · ${escapeHtml(report.deliberation_id)}</title>
<style>
  :root { color-scheme: dark; }
  * { box-sizing: border-box; }
  body { margin: 0; font: 15px/1.5 ui-monospace, "Cascadia Code", Menlo, monospace;
    background: radial-gradient(1200px 800px at 20% -10%, #1a1442, #0a0716 60%);
    color: #e8e6f5; padding: 2rem; }
  h1 { font-size: 1.3rem; margin: 0 0 .2rem; letter-spacing: .04em; }
  .q { color: #b6aee0; margin: 0 0 1.2rem; max-width: 60ch; }
  .summary { display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.2rem; }
  .summary span { padding: .3rem .7rem; border-radius: .5rem; background: #ffffff10; }
  .cohesion { height: .5rem; background: #ffffff18; border-radius: 1rem; overflow: hidden; max-width: 420px; }
  .cohesion > span { display: block; height: 100%; background: linear-gradient(90deg,#7c5cff,#25d0c0); }
  .grid { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); }
  .card { border: 1px solid hsl(var(--hue) 60% 45% / .5); border-radius: .8rem;
    padding: 1rem; background: hsl(var(--hue) 50% 20% / .35); }
  .card header { display: flex; justify-content: space-between; align-items: center; gap: .5rem; }
  .pill { font-size: .7rem; letter-spacing: .08em; padding: .15rem .5rem; border-radius: 1rem;
    background: hsl(var(--hue) 70% 50% / .3); }
  .card code { color: #cdbbff; }
  .claim { margin: .6rem 0; }
  .meter { height: .4rem; background: #ffffff14; border-radius: 1rem; overflow: hidden; }
  .meter > span { display: block; height: 100%; background: linear-gradient(90deg,#ff5470,#25d0c0); }
  dl { display: flex; gap: 1rem; margin: .6rem 0 0; flex-wrap: wrap; }
  dl div { text-align: center; } dt { font-size: .65rem; color: #9d95c9; } dd { margin: 0; }
  .dissent { margin-top: .5rem; color: #ff8fa3; font-size: .85rem; }
  .roster { list-style: none; padding: 0; margin: 1.5rem 0 0; }
  .roster li { display: grid; grid-template-columns: 8ch 1fr 10rem 4rem; gap: .6rem; align-items: center; }
  .roster .bar { height: .5rem; background: #ffffff14; border-radius: 1rem; overflow: hidden; }
  .roster .bar > span { display: block; height: 100%; background: linear-gradient(90deg,#7c5cff,#25d0c0); }
  .roster em { text-align: right; color: #b6aee0; } .roster span { color: #9d95c9; }
  h2 { font-size: .95rem; letter-spacing: .08em; color: #cdbbff; margin-top: 2rem; }
</style>
</head>
<body>
  <h1>QUORUMFORGE · ${escapeHtml(report.deliberation_id)}</h1>
  <p class="q">${escapeHtml(report.question)}</p>
  <div class="summary">
    <span>claims ${s.total_claims}</span>
    <span>consensus ${s.consensus}</span>
    <span>contested ${s.contested}</span>
    <span>split ${s.split}</span>
    <span>unsupported ${s.unsupported}</span>
  </div>
  <div class="cohesion"><span style="width:${(s.cohesion * 100).toFixed(1)}%"></span></div>
  <p class="q">council cohesion ${(s.cohesion * 100).toFixed(1)}%</p>
  <h2>VERDICTS</h2>
  <div class="grid">
${verdictCards}
  </div>
  <h2>COUNCIL ROSTER</h2>
  <ul class="roster">
${roster}
  </ul>
</body>
</html>
`;
}
