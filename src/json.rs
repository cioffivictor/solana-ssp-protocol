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

    /// Borrow this value as an array slice, if it is an array.
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }

    /// Borrow this value as a string, if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Interpret this value as an `f64`, if it is a number.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Interpret this value as a bool, if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// An error produced while parsing JSON text.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON error at byte {}: {}", self.position, self.message)
    }
