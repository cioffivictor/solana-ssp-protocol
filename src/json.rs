//! A compact, dependency-free JSON reader and writer.
//!
//! QuorumForge accepts and emits JSON without pulling in `serde`. This module
//! provides just enough of a JSON document model to parse deliberation files
//! and serialise deterministic reports. It is intentionally small: it supports
//! objects, arrays, strings, numbers (as `f64`), booleans, and null.
//!
//! The writer is stable: object keys are emitted in insertion order, so the
//! same in-memory document always serialises to byte-identical output. This is
//! a load-bearing property for QuorumForge's reproducible bundles.

use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// Object entries preserve insertion order for deterministic output.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Borrow this value as an object entry list, if it is an object.
    pub fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Obj(entries) => Some(entries),
            _ => None,
        }
    }

    /// Look up a key within an object value.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

