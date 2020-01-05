//! # QuorumForge
//!
//! QuorumForge is a **multi-agent evidence adjudication engine**. It ingests a
//! recorded deliberation — a council of weighted agents taking positions on a
//! set of claims, with citations and confidences — and turns it into a
//! deterministic verdict: which claims reached consensus, which are contested,
//! which merely lean one way, and which lack the evidence to conclude anything.
//!
//! It is *not* an agent orchestrator. QuorumForge does not run agents, call
