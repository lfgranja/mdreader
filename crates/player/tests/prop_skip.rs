//! T013 — BLOCKING US1 TEST: property `prop_skip_section_fallback`
//! (Feature 004-ressoa-dogfood-build / task T013).
//!
//! Invariants under test (tasks.md T013, verbatim):
//! "Property `prop_skip_section_fallback`: skip lands on
//! next/previous section boundary; zero sections ⇒ exactly
//! ±10 s; never mid-word anchor lost; clamp at doc ends."
//!
//! Bind only to the public surface:
//! [`player::skip::skip_section`] and
//! [`doc_structure::Heading`] — the boundary provider the
//! adapter/player already consumes (FR-024a / contracts/
//! ingest.md). No local re-declaration of any contract type.
//!
//! FAILS before `skip_section` implements the boundary
//! contract — acceptable red state for the blocking gate.

use proptest::prelude::*;

use doc_structure::Heading;
use player::skip::skip_section;
use player::transport::SkipDirection;

/// Generated heading positions, ascending, on the
/// quarter-second grid — the T061b provider shape. The
/// position arguments are drawn from the same grid so the
/// generated (position, headings) pairs are dense in the
/// boundary-crossing cases the invariant cares about.
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every quarter-second count at 14 400 (< 2^14), far below f64's exact-integer range (2^53); the grid position converts exactly"
)]
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// A generated document duration that always covers every
/// generated position (so the clamp bounds are exercised at
/// the document end, not before it).
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every quarter-second count at 14 400 (< 2^14), far below f64's exact-integer range (2^53); the grid position converts exactly"
)]
#[expect(
    clippy::suboptimal_flops,
    reason = "the +1.0 s guard stays separate from the exact ×0.25 grid product so the duration remains the exact binary fraction the clamp assertions compare against"
)]
fn duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25 + 1.0)
}

/// An ascending, de-duplicated heading list on the
/// quarter-second grid.
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every quarter-second count at 14 400 (< 2^14), far below f64's exact-integer range (2^53); the grid position converts exactly"
)]
fn headings_strategy() -> impl Strategy<Value = Vec<Heading>> {
    proptest::collection::vec(
        (
            0u64..=14_400,
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
                position_secs: quarters as f64 * 0.25,
                title,
            })
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Forward skip lands on the NEXT heading boundary strictly
    /// after the position. When the position is past the last
    /// heading (no boundary ahead) the skip clamps to the
    /// document end — the ±10 s fallback is a ZERO-SECTIONS
    /// rule only, never a non-empty-list fallback (T013).
    #[expect(
        clippy::float_cmp,
        reason = "heading positions are exact quarter-second grid values and the landing must equal one of them; == is the correct assertion and a tolerance would admit a mid-anchor landing"
    )]
    #[test]
    fn prop_forward_skip_lands_on_next_boundary(
        position in position_strategy(),
        headings in headings_strategy(),
    ) {
        prop_assume!(!headings.is_empty());
        let duration = 20_000.0_f64.max(position + 1.0);
        let landing = skip_section(position, &headings, SkipDirection::Forward, duration);
        let next_ahead = headings.iter().any(|h| h.position_secs > position);
        if next_ahead {
            prop_assert!(
                headings.iter().any(|h| h.position_secs == landing && h.position_secs > position),
                "forward skip from {} landed at {} which is not the next heading boundary (headings {:?})",
                position, landing, headings
            );
        } else {
            prop_assert!(
                (landing - duration).abs() < 1e-9,
                "forward skip from {} past the last heading must clamp to the document end {}, got {}",
                position, duration, landing
            );
        }
    }

    /// Backward skip lands on a heading boundary at or before
    /// the position. When the position sits exactly on a
    /// heading (or before the first), the landing must be a
    /// boundary STRICTLY before the position — a skip that
    /// does not move is not a skip. When the position is
    /// before the first heading the landing is the 0.0 floor.
    #[test]
    fn prop_backward_skip_lands_on_previous_boundary(
        position in position_strategy(),
        headings in headings_strategy(),
    ) {
        prop_assume!(!headings.is_empty());
        let duration = 20_000.0_f64.max(position + 1.0);
        let landing = skip_section(position, &headings, SkipDirection::Backward, duration);
        let expected = headings
            .iter()
            .rfind(|h| h.position_secs < position)
            .map_or(0.0, |h| h.position_secs);
        prop_assert!(
            (landing - expected).abs() < 1e-9,
            "backward skip from {} landed at {} but the previous-boundary contract requires {} (headings {:?})",
            position, landing, expected, headings
        );
    }

    /// Zero sections ⇒ exactly ±10.0 s (clamped at the 0.0
    /// floor and the document duration).
    #[test]
    fn prop_zero_sections_exact_ten_second_fallback(
        position in position_strategy(),
        duration in duration_strategy(),
    ) {
        prop_assume!(duration >= position);
        let empty: Vec<Heading> = vec![];
        let forward = skip_section(position, &empty, SkipDirection::Forward, duration);
        let backward = skip_section(position, &empty, SkipDirection::Backward, duration);
        let expected_forward = (position + 10.0).min(duration);
        let expected_backward = (position - 10.0).max(0.0);
        prop_assert!(
            (forward - expected_forward).abs() < 1e-9,
            "zero-section forward skip from {} within {} must land exactly +10 s (clamped): got {}, expected {}",
            position, duration, forward, expected_forward
        );
        prop_assert!(
            (backward - expected_backward).abs() < 1e-9,
            "zero-section backward skip from {} within {} must land exactly -10 s (clamped): got {}, expected {}",
            position, duration, backward, expected_backward
        );
    }

    /// Every skip lands inside [0.0, duration] — the clamp at
    /// the document ends. The landing is also finite.
    #[test]
    fn prop_skip_clamped_at_document_ends(
        position in position_strategy(),
        headings in headings_strategy(),
        duration in duration_strategy(),
    ) {
        prop_assume!(duration >= position);
        for direction in [SkipDirection::Forward, SkipDirection::Backward] {
            let landing = skip_section(position, &headings, direction, duration);
            prop_assert!(
                landing.is_finite(),
                "skip landing must be finite, got {}",
                landing
            );
            prop_assert!(
                landing >= 0.0,
                "skip landing {} below the 0.0 floor (clamp violated)",
                landing
            );
            prop_assert!(
                landing <= duration,
                "skip landing {} beyond the document duration {} (clamp violated)",
                landing, duration
            );
        }
    }

    /// Mid-word anchor preservation: a skip never lands the
    /// resume anchor at an arbitrary interior position. The
    /// observable form of this invariant for a
    /// paragraph/heading-granularity adapter is that the
    /// landing is ALWAYS either a heading boundary, the 0.0
    /// floor, the duration ceiling, or the exact ±10 s
    /// fallback point (zero sections) — never anything else.
    #[test]
    fn prop_skip_never_lands_mid_anchor(
        position in position_strategy(),
        headings in headings_strategy(),
        duration in duration_strategy(),
    ) {
        prop_assume!(duration >= position);
        for direction in [SkipDirection::Forward, SkipDirection::Backward] {
            let landing = skip_section(position, &headings, direction, duration);
            let on_boundary = headings
                .iter()
                .any(|h| (h.position_secs - landing).abs() < 1e-9);
            let on_floor = landing.abs() < 1e-9;
            let on_ceiling = (landing - duration).abs() < 1e-9;
            let on_fallback = headings.is_empty()
                && ((landing - (position + 10.0).min(duration)).abs() < 1e-9
                    || (landing - (position - 10.0).max(0.0)).abs() < 1e-9);
            prop_assert!(
                on_boundary || on_floor || on_ceiling || on_fallback,
                "skip from {} landed at {} — not a heading boundary, floor, ceiling, or the zero-section fallback point (mid-anchor violation)",
                position, landing
            );
        }
    }
}
