//! Human- and machine-readable reports.
//!
//! Two renderers share one source of truth (the [`Adjudication`]):
//!
//! * [`render_text`] produces a fixed-width console report with a headline,
//!   a per-claim table, and a dissent roster.
//! * [`render_json`] produces the structured report the TypeScript council
//!   viewer consumes. The JSON report is a superset of the bundle summary,
//!   enriched with the per-agent influence breakdown.
//!
//! Both renderers are deterministic and free of timestamps or environment
//! details, so their output can be committed as golden files.

use crate::adjudicate::{Adjudication, Outcome, Verdict};
use crate::json::{self, Json};
