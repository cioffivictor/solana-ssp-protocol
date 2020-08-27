// Type definitions mirroring the `quorumforge.report.v1` JSON schema emitted by
// `quorumforge adjudicate --format json`. Keeping these in one place lets the
// renderer stay fully typed without any runtime schema library.

export type Outcome = "consensus" | "contested" | "split" | "unsupported";

export interface AgentView {
  id: string;
  name: string;
  role: string;
