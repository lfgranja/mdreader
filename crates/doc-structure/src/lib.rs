//! Doc-structure crate — the ordered-headings provider
//! (US4, T061c; spec anchors FR-028d and FR-038).
//!
//! A narration document is resolved through its ordered
//! headings: each heading is a timeline position in
//! seconds plus the title of the section it opens. The
//! headings are a runtime capability in this feature —
//! FR-024 defers the parser — so [`Document::new`]
//! accepts the parsed list and normalizes any input
//! order (stable ascending sort by
//! [`Heading::position_secs`]), and every resolution
//! query is a pure function of the normalized list:
//! [`Document::headings_before`] (the contiguous
//! prefix at or before a position),
//! [`Document::nearest_boundary_at_or_before`] (the
//! greatest boundary at or before a position) and
//! [`Document::reanchor`] (the FR-038 re-anchor
//! point — the nearest boundary, else exactly `0.0`).
//!
//! Every capability is also CLI-drivable (Principle
//! II; `constitution.md`): the [`cli`] module is the
//! text-I/O protocol entry point the `doc-structure`
//! binary forwards to, with the machine-readable JSON
//! form as the default for scripting.

pub mod cli;

use serde::{Deserialize, Serialize};

/// One ordered heading: the timeline position of a
/// section boundary and the title of the section it
/// opens.
///
/// The title is an identifier, never document body
/// text — the same identifier-pattern rule the
/// diagnostic record follows. The position is a
/// timeline offset in seconds; it may be negative (a
/// heading before the narration timeline's origin)
/// and may repeat (two sections opening at the same
/// position), both of which the ordering handles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading {
    /// The heading's position on the narration
    /// timeline, in seconds.
    pub position_secs: f64,
    /// The title of the section this heading opens —
    /// an identifier, never document body text.
    pub title: String,
}

/// A document resolved through its ordered headings:
/// the normalized, ascending-ordered list every
/// resolution query reads.
///
/// [`Document::new`] sorts the input stably ascending
/// by [`Heading::position_secs`], so the resolution
/// invariants hold for any input order, including
/// unsorted lists and duplicate positions: ties keep
/// their input order (the stable sort), and every
/// query is a pure function of the normalized list.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// The ordered headings, ascending by
    /// `position_secs` (stable).
    headings: Vec<Heading>,
}

impl Document {
    /// A document over `headings`, normalized to
    /// ascending order by `position_secs`.
    ///
    /// The sort is stable, so headings that share a
    /// position keep their input order — the
    /// invariants hold for any input order, including
    /// unsorted lists and duplicate positions.
    #[must_use]
    pub fn new(mut headings: Vec<Heading>) -> Self {
        headings.sort_by(|left, right| left.position_secs.total_cmp(&right.position_secs));
        Self { headings }
    }

    /// The ordered headings, ascending by
    /// `position_secs`.
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    /// The contiguous prefix of the ordered headings at
    /// or before `pos` — exactly the first `k`
    /// headings for some `k`, every one with
    /// `position_secs <= pos`.
    ///
    /// The prefix only grows as `pos` advances: the
    /// boundary set never loses a heading, and the
    /// earlier prefix is a prefix of the later one.
    #[must_use]
    pub fn headings_before(&self, pos: f64) -> &[Heading] {
        // The ordered list is partitioned with respect
        // to `position_secs <= pos`, so the headings at
        // or before `pos` are exactly the ones before
        // the partition point.
        let count = self
            .headings
            .partition_point(|heading| heading.position_secs <= pos);
        &self.headings[..count]
    }

    /// The nearest boundary at or before `pos`: the
    /// greatest heading with `position_secs <= pos`, or
    /// `None` when every heading is after `pos`
    /// (vacuously so for an empty document).
    ///
    /// The result is maximal: no heading falls strictly
    /// between the returned boundary and `pos`.
    #[must_use]
    pub fn nearest_boundary_at_or_before(&self, pos: f64) -> Option<&Heading> {
        // The nearest boundary is the last heading of
        // the at-or-before prefix, when that prefix is
        // non-empty; an empty prefix means no boundary
        // is at or before `pos`.
        let count = self
            .headings
            .partition_point(|heading| heading.position_secs <= pos);
        if count == 0 {
            return None;
        }
        self.headings.get(count - 1)
    }

    /// The FR-038 re-anchor point for `pos`: the
    /// nearest boundary at or before `pos`, else
    /// exactly `0.0` — never a value strictly between
    /// two boundaries, and never an arbitrary offset.
    #[must_use]
    pub fn reanchor(&self, pos: f64) -> f64 {
        self.nearest_boundary_at_or_before(pos)
            .map_or(0.0, |heading| heading.position_secs)
    }
}
