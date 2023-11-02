// Type definitions mirroring the `quorumforge.report.v1` JSON schema emitted by
// `quorumforge adjudicate --format json`. Keeping these in one place lets the
// renderer stay fully typed without any runtime schema library.

export type Outcome = "consensus" | "contested" | "split" | "unsupported";

export interface AgentView {
  id: string;
  name: string;
  role: string;
  weight: number;
  influence: number;
}

export interface VerdictView {
  claim: string;
  text: string;
  topic: string;
  normalized: string;
  outcome: Outcome;
  affirmed: boolean;
  polarity: number;
  decisive_mass: number;
  support_mass: number;
  contradiction_mass: number;
  dissent_ratio: number;
  supporters: number;
  dissenters: number;
  abstentions: number;
  citations: number;
  majority_agents: string[];
  minority_agents: string[];
}

export interface ReportSummary {
  total_claims: number;
  consensus: number;
  contested: number;
  split: number;
  unsupported: number;
  cohesion: number;
}

export interface Policy {
  consensus_threshold: number;
  dissent_ceiling: number;
  minimum_mass: number;
}

export interface Report {
  schema: string;
  deliberation_id: string;
  question: string;
  summary: ReportSummary;
  policy: Policy;
  agents: AgentView[];
  verdicts: VerdictView[];
