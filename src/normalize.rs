//! Claim normalization.
//!
//! Two agents rarely phrase the same proposition identically. The normalizer
//! reduces claim text to a canonical form so that near-duplicate claims can be
//! recognised and, optionally, merged. Normalization is intentionally
//! conservative: it lower-cases, collapses whitespace, strips a small set of
//! leading hedges and trailing punctuation, and folds a handful of common
//! contractions. It does *not* attempt semantic paraphrase detection — that
//! would make verdicts non-deterministic and hard to audit.

use crate::model::{Claim, Deliberation};
use std::collections::BTreeMap;

