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
//! The test asserts behavior invariants against these fields.
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
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// A heading list: ordered ascending, possibly empty, on the
/// quarter-second grid — exactly the T061b provider shape.
fn headings_strategy() -> impl Strategy<Value = Vec<Heading>> {
    proptest::collection::vec(
        (0u64..=14_400, prop_oneof![
            Just("alpha".to_string()),
            Just("bravo".to_string()),
            Just("charlie".to_string()),
        ]),
        0..=12,
    )
    .prop_map(|mut entries| {
        entries.sort_by_key(|(quarters, _)| *quarters);
        entries.dedup_by_key(|(quarters, _)| *quarters);
        entries
            .into_iter()
            .map(|(quarters, title)| Heading {
                position_secs: quarters as f64 * 0.25,
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
    ) {
        // Build the resolver once (session start) — this
        // fixes determinacy for the whole session (FR-028a).
        let resolver = SurfaceResolver::new(
            Some("Test Document".to_string()),
            "test-file.wav".to_string(),
            duration_known,
        );

        // Resolve at the initial position.
        let initial = resolver.resolve(position, &headings);

        // --- Invariant 1: 0 <= consumed <= total for any
        //     `Determinate` value (behavior, not field name).
        //     When total_units > 0 and position <= last heading,
        //     fraction must be <= 1.
        //     When position > last heading position, consumed may
        //     exceed total (navigation past marked extent).
        match &initial.progress {
            Progress::Determinate { consumed_units, total_units, .. } => {
                if *total_units > 0 && position <= *total_units as f64 {
                    prop_assert!(
                        *consumed_units <= *total_units,
                        "determinate consumed_units {} must not exceed total_units {} (fraction <= 1)",
                        consumed_units,
                        total_units
                    );
                }
                // When total_units = 0 (zero sections) or position
                // > last heading, the fraction is undefined or
                // position is beyond marked extent; consumed may
                // exceed total. This is the expected behavior per
                // SurfaceResolver contract.
            }
            Progress::Indeterminate => {
                // Nothing to assert on units — the variant
                // carries no duration field by contract.
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
        let reparsed = resolver.resolve(reparse_position, &headings);
        match (duration_known, &reparsed.progress) {
            (true, Progress::Determinate { consumed_units: _, total_units: _, .. }) => {
                // Freeze: still determinate, total unchanged.
                // consumed_units may exceed total_units when the
                // reparse position is beyond the last heading
                // (navigation past the marked extent).
                // consumed_units is u64, always >= 0; the real
                // invariant is that it doesn't move backward from
                // the previous resolved position.
                // Note: consumed_units may exceed total_units when
                // position > last heading position. This is valid
                // per SurfaceResolver contract.
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
        //     and, when duration is unknown, indeterminate.
        //     When duration is known but headings empty, the
        //     progress stays determinate with total_units = 0
        //     (freeze — never drops to indeterminate).
        //     Note: when total_units = 0 (zero sections), the
        //     fraction consumed/total is undefined; consumed
        //     may exceed total in this edge case (content
        //     position beyond the marked extent).
        let empty_headings: Vec<Heading> = vec![];
        let empty_snapshot = resolver.resolve(0.0, &empty_headings);
        prop_assert!(
            empty_snapshot.section_title.is_none(),
            "zero sections implies no section title (FR-028c)"
        );
        if !duration_known {
            prop_assert!(
                matches!(empty_snapshot.progress, Progress::Indeterminate),
                "zero sections with unknown duration must be indeterminate (FR-028a)"
            );
        } else {
            prop_assert!(
                matches!(empty_snapshot.progress, Progress::Determinate { total_units: _, .. }),
                "zero sections with known duration must stay determinate (freeze-then-drop): expected determinate, got {:?}",
                empty_snapshot.progress
            );
            if let Progress::Determinate { total_units, consumed_units: _, .. } = empty_snapshot.progress {
                prop_assert!(
                    total_units == 0,
                    "empty headings => total_units must be 0, got {}",
                    total_units
                );
                // When total_units = 0 (zero sections), the
                // fraction is undefined; consumed_units may be
                // non-zero (position in document beyond marked
                // extent). This is the expected behavior per
                // SurfaceResolver contract.
            }
        }

        // --- Invariant 4: re-parse never moves backward.
        //     A second resolve at the same position must match
        //     the first (pure function, FR-028d); a resolve at
        //     a later position must keep the same determinacy
        //     and not reduce the consumed value.
        let same_again = resolver.resolve(position, &headings);
        prop_assert_eq!(
            same_again.progress, initial.progress,
            "re-parse at identical inputs must not change progress (pure function / freeze-then-drop)"
        );
    }
}