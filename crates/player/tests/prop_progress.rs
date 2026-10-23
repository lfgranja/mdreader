//! T006 — BLOCKING FOUNDATIONAL TEST: property `prop_progress_fraction`
//! (Feature 004-ressoa-dogfood-build / task T006).
//!
//! Invariants under test (tasks.md T006, verbatim):
//! "Property `prop_progress_fraction`: `0 ≤ consumed ≤ total`;
//! determinate fixed at session start, never flips mid-session
//! (freeze-then-drop); zero sections ⇒ indeterminate; re-parse
//! never moves backward — freeze-then-drop."
//!
//! The progress form uses the content-offset shape (T005):
//! `Determinate { consumed_units, total_units, k, n }` with
//! u64/u64/u32/u32 fields. `Indeterminate` carries no fields.
//! The test asserts behavior invariants against these fields:
//! `total_units` is the parsed-content unit count fixed at
//! session start (one unit = one sentence-level segment the
//! adapter's chunk split emits, FR-013) and `consumed_units`
//! is the session position projected onto that unit grid —
//! never the position rendered as units and never an
//! audio-duration estimate.
//!
//! Spec anchors: FR-028a (progress fixed at session start,
//! never estimated, never switches), FR-028c (section is the
//! most-recent-passed heading, none before first / for empty),
//! and the freeze-then-drop contract (re-parse on boundary
//! cross / resume / skip / re-anchor never moves the progress
//! backward; once determinate, always determinate).
//!
//! FAILS before `Progress` resolves — acceptable red state for
//! the blocking gate.

use proptest::prelude::*;

use player::surface::{Heading, Progress, SurfaceResolver};

/// A generated position on the quarter-second grid — the
/// same grid the surface resolver uses.
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u16..=14_400).prop_map(|quarters| f64::from(quarters) * 0.25)
}

/// A heading list: ordered ascending, possibly empty, on the
/// quarter-second grid — exactly the T061b provider shape.
fn headings_strategy() -> impl Strategy<Value = Vec<Heading>> {
    proptest::collection::vec(
        (
            0u16..=14_400,
            prop_oneof![
                Just("alpha".to_string()),
                Just("bravo".to_string()),
                Just("charlie".to_string()),
            ],
        ),
        0..=12,
    )
    .prop_map(|mut entries| {
        entries.sort_by_key(|(quarters, _)| *quarters);
        entries.dedup_by_key(|(quarters, _)| *quarters);
        entries
            .into_iter()
            .map(|(quarters, title)| Heading {
                position_secs: f64::from(quarters) * 0.25,
                title,
            })
            .collect()
    })
}

/// A boolean fixing determinacy at session start: true =
/// the session knew its total, false = unknown (stays
/// indeterminate for the whole session, FR-028a).
fn duration_known_strategy() -> impl Strategy<Value = bool> {
    prop_oneof![Just(true), Just(false)]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T006: `prop_progress_fraction` — the core progress
    /// fraction invariants.
    ///
    /// - `Determinate`: consumed_units >= 0, total_units >= 0,
    ///   consumed_units <= total_units.
    /// - `Indeterminate`: never carries units (no backward
    ///   move from determinate).
    /// - Zero headings => indeterminate progress when duration
    ///   was unknown at start; when known, the total is 0 but
    ///   the variant stays determinate (freeze-then-drop).
    /// - Re-parse (re-resolve at same / later position) never
    ///   moves the progress backward: a determinate session
    ///   stays determinate; an indeterminate session stays
    ///   indeterminate.
    #[test]
    fn prop_progress_fraction(
        duration_known in duration_known_strategy(),
        headings in headings_strategy(),
        position in position_strategy(),
        seed in any::<u8>(),
    ) {
        // Build the resolver once (session start) — this
        // fixes the parsed-content totals (the unit total
        // and its projection basis) and the determinacy for
        // the whole session (FR-028a / FR-013). The unit
        // total is the chunk split's segment count: the
        // fixture derives it from the seed so a document's
        // total is a content fact, never an estimate.
        let unit_count = seed % 61 + 4; // 4..=64 segments — exact in f64
        let total_units = u64::from(unit_count);
        let unit_secs = 0.25; // the quarter-second grid the fixture uses
        // The session-start total in seconds, from the small
        // (≤64) segment count so the conversion is exact.
        let total_secs = f64::from(unit_count) * unit_secs;
        let resolver = SurfaceResolver::new(
            Some("Test Document".to_string()),
            "test-file.wav".to_string(),
            total_units,
            total_secs,
            duration_known,
        );

        // Resolve at the initial position: the caller projects
        // the position onto the parsed-content offset — the
        // resolver never derives it from `position_secs`
        // itself (FR-013).
        let initial = resolver.resolve(position, resolver.consumed_units_at(position), &headings);

        // --- Invariant 1: 0 <= consumed <= total for any
        //     `Determinate` value (behavior, not field name).
        //     The consumed offset is the position's
        //     projection onto the unit grid, clamped to the
        //     session-start total, so the fraction never
        //     exceeds 1 whatever the position.
        match &initial.progress {
            Progress::Determinate {
                consumed_units,
                total_units: matched_total,
                ..
            } => {
                prop_assert!(
                    *consumed_units <= *matched_total,
                    "determinate consumed_units {} must not exceed total_units {} (fraction <= 1)",
                    consumed_units,
                    matched_total
                );
                // The total is the session-start parsed-content
                // unit count, fixed at construction — never the
                // greatest heading position and never estimated.
                prop_assert_eq!(
                    *matched_total,
                    total_units,
                    "the determinate total must be the session-start parsed-content unit count (FR-013)"
                );
                prop_assert_eq!(
                    *consumed_units,
                    resolver.consumed_units_at(position),
                    "the consumed offset must be the position's parsed-content projection (FR-013)"
                );
            }
            Progress::Indeterminate => {
                // Nothing to assert on units — the variant
                // carries no unit field by contract.
            }
        }

        // --- Invariant 2: once `Determinate` at session
        //     start, the session never becomes `Indeterminate`
        //     on re-parse (freeze-then-drop). The same resolver
        //     is reused; only the input headings/position
        //     change, never the start-time determinacy.
        if duration_known {
            prop_assert!(
                matches!(initial.progress, Progress::Determinate { .. }),
                "a session that knew its total at start must resolve determinate at position {}",
                position
            );
        }

        // Re-resolve at a later (or same) position —
        // simulating the freeze-then-drop re-parse.
        let reparse_position = position + 5.0; // any later position
        let reparsed =
            resolver.resolve(reparse_position, resolver.consumed_units_at(reparse_position), &headings);
        match (duration_known, &reparsed.progress) {
            (true, Progress::Determinate { consumed_units, total_units: reparsed_total, .. }) => {
                // Freeze: still determinate, the frozen total
                // unchanged across the re-parse.
                prop_assert_eq!(
                    *reparsed_total,
                    total_units,
                    "the re-parse moved the frozen total (FR-013: freeze-then-drop)"
                );
                // No backward jump: the re-parse position is
                // later, so the projected consumed offset never
                // decreases.
                prop_assert!(
                    *consumed_units >= resolver.consumed_units_at(position),
                    "a later position projected backward (FR-013: no backward jump)"
                );
            }
            (true, Progress::Indeterminate) => {
                prop_assert!(
                    false,
                    "freeze-then-drop violated: a determinate session flipped to Indeterminate on re-parse at {}",
                    reparse_position
                );
            }
            (false, Progress::Indeterminate) => {
                // Drop (stays indeterminate) — no flip.
            }
            (false, Progress::Determinate { .. }) => {
                prop_assert!(
                    false,
                    "an indeterminate session must not become determinate on re-parse (freeze-then-drop: total never estimated)",
                );
            }
        }

        // --- Invariant 3: zero headings => no section title
        //     and, when the totals were unknown at start,
        //     indeterminate. When the totals were known but
        //     the heading list is empty, the progress stays
        //     determinate with the frozen unit total
        //     (freeze — never drops to indeterminate): the
        //     unit total is the chunk split's segment count,
        //     a content fact independent of the section
        //     list.
        let empty_headings: Vec<Heading> = vec![];
        let empty_snapshot =
            resolver.resolve(0.0, resolver.consumed_units_at(0.0), &empty_headings);
        prop_assert!(
            empty_snapshot.section_title.is_none(),
            "zero sections implies no section title (FR-028c)"
        );
        if duration_known {
            prop_assert!(
                matches!(empty_snapshot.progress, Progress::Determinate { .. }),
                "zero sections with known duration must stay determinate (freeze-then-drop): expected determinate, got {:?}",
                empty_snapshot.progress
            );
            if let Progress::Determinate { total_units: empty_total, .. } = empty_snapshot.progress {
                // Zero HEADINGS do not zero the total: the total is
                // the chunk split's segment count, a content fact
                // independent of the section list. The section
                // count `n` is 0, so no section title resolves
                // (FR-028c).
                prop_assert_eq!(
                    empty_total,
                    total_units,
                    "empty headings must not move the session-start unit total (FR-013)"
                );
                prop_assert_eq!(empty_snapshot.section_title, None);
            }
        } else {
            prop_assert!(
                matches!(empty_snapshot.progress, Progress::Indeterminate),
                "zero sections with unknown duration must be indeterminate (FR-028a)"
            );
        }

        // --- Invariant 4: re-parse never moves backward.
        //     A second resolve at the same position must match
        //     the first (pure function, FR-028d); a resolve at
        //     a later position must keep the same determinacy
        //     and not reduce the consumed value.
        let same_again =
            resolver.resolve(position, resolver.consumed_units_at(position), &headings);
        prop_assert_eq!(
            same_again.progress, initial.progress,
            "re-parse at identical inputs must not change progress (pure function / freeze-then-drop)"
        );
    }
}
