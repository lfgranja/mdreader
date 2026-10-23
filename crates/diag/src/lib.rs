//! Diag crate — JSONL diagnostic record.

/// Append-only JSONL diagnostic-record writer (T073).
pub mod writer;

/// The no-prose / closed-set scanners (T074).
pub mod scan;

/// The read-only scan CLI (T074).
pub mod cli;
