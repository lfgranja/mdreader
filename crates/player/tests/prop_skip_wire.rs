//! T063 — BLOCKING US1 TEST: property `prop_skip_wire`
//! (Feature 004-ressoa-dogfood-build / task T063).
//!
//! Invariants under test (tasks.md T063, verbatim):
//! "drive session.execute(ShellVerb::SkipForward/Backward) over
//! arbitrary documents/positions and assert the landing is the
//! next/previous section boundary (zero sections ⇒ exactly ±10 s;
//! clamp at doc ends) — NOT ±5.0 s"
//!
//! FAILS before T065 wires `skip_section` into the live skip path
//! (the current path displaces ±5.0 s; the contract requires ±10 s
//! for the zero-sections case and a boundary jump otherwise).

use proptest::prelude::*;

use player::cli::{PlaybackSession, ShellVerb};
use player::surface::Heading;

fn position_strategy() -> impl Strategy<Value = f64> {
    (0u16..=3600).prop_map(f64::from)
}

fn document_strategy() -> impl Strategy<Value = (f64, Vec<Heading>)> {
    (
        10u16..=3600,
        proptest::collection::vec(
            (0u16..=3600, "[a-z]{1,8}"),
            0..=12,
        ),
    )
        .prop_map(|(duration, entries)| {
            let duration = f64::from(duration);
            let mut headings = entries
                .into_iter()
                .filter(|(position, _)| f64::from(*position) <= duration)
                .map(|(position_secs, title)| Heading {
                    position_secs: f64::from(position_secs),
                    title,
                })
                .collect::<Vec<_>>();
            headings.sort_by(|left, right| {
                left.position_secs.total_cmp(&right.position_secs)
            });
            headings.dedup_by(|left, right| left.position_secs == right.position_secs);
            (duration, headings)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Zero-sections forward skip lands at exactly +10.0 s (clamped
    /// to the document duration). The current ±5.0 s displacement
    /// fails this test — the red state for T065.
    #[test]
    fn prop_skip_forward_zero_sections_exact_ten_seconds(duration in 10.0..=1000.0_f64) {
        let mut session = PlaybackSession::with_duration(duration);
        let _ = session.execute(ShellVerb::Play);
        let snapshot = session.execute(ShellVerb::SkipForward).unwrap();
        let expected = 10.0_f64.min(duration);
        prop_assert!(
            (snapshot.position_secs - expected).abs() < 1e-9,
            "zero-section forward skip from 0.0 within {} must land exactly +10 s: got {}",
            duration,
            snapshot.position_secs,
        );
    }

    /// Zero-sections backward skip from 0.0 clamps at the floor.
    #[test]
    fn prop_skip_backward_zero_sections_clamps_at_zero(duration in 10.0..=1000.0_f64) {
        let mut session = PlaybackSession::with_duration(duration);
        let _ = session.execute(ShellVerb::Play);
        let snapshot = session.execute(ShellVerb::SkipBackward).unwrap();
        prop_assert!(
            snapshot.position_secs.abs() < 1e-9,
            "zero-section backward skip from 0.0 must clamp at 0.0: got {}",
            snapshot.position_secs,
        );
    }

    /// Skip landing is always finite and within [0.0, duration].
    #[test]
    fn prop_skip_landing_within_bounds(duration in 1.0..=1000.0_f64) {
        let mut session = PlaybackSession::with_duration(duration);
        let _ = session.execute(ShellVerb::Play);
        for verb in [ShellVerb::SkipForward, ShellVerb::SkipBackward] {
            let snapshot = session.execute(verb).unwrap();
            prop_assert!(
                snapshot.position_secs.is_finite(),
                "skip landing must be finite, got {}",
                snapshot.position_secs,
            );
            prop_assert!(
                snapshot.position_secs >= 0.0,
                "skip landing {} below the 0.0 floor",
                snapshot.position_secs,
            );
            prop_assert!(
                snapshot.position_secs <= duration,
                "skip landing {} beyond the document duration {}",
                snapshot.position_secs,
                duration,
            );
        }
    }

    /// Across generated sectioned documents and positions, live
    /// SkipForward lands on the first strictly later heading, the
    /// document end when no heading remains, or the +10 s fallback
    /// only when the document has no sections.
    #[test]
    fn prop_skip_forward_uses_next_boundary(
        (duration, headings) in document_strategy(),
        position in position_strategy(),
    ) {
        let position = position.min(duration);
        let mut session = PlaybackSession::with_duration(duration);
        session.set_document_structure(duration, headings.clone());
        let _ = session.execute(ShellVerb::Play);
        session.on_audible_progress(position);
        let snapshot = session.execute(ShellVerb::SkipForward).unwrap();
        let expected = headings
            .iter()
            .find(|heading| heading.position_secs > position)
            .map_or_else(
                || {
                    if headings.is_empty() {
                        (position + 10.0).min(duration)
                    } else {
                        duration
                    }
                },
                |heading| heading.position_secs,
            );
        prop_assert!(
            (snapshot.position_secs - expected).abs() < 1e-9,
            "forward skip from {} with headings {:?} must land at {}, got {}",
            position,
            headings,
            expected,
            snapshot.position_secs,
        );
    }

    /// Across generated documents and positions, live SkipBackward
    /// lands on the previous strictly earlier heading, the floor if
    /// no earlier section remains, or −10 s only with no sections.
    #[test]
    fn prop_skip_backward_uses_previous_boundary(
        (duration, headings) in document_strategy(),
        position in position_strategy(),
    ) {
        let position = position.min(duration);
        let mut session = PlaybackSession::with_duration(duration);
        session.set_document_structure(duration, headings.clone());
        let _ = session.execute(ShellVerb::Play);
        session.on_audible_progress(position);
        let snapshot = session.execute(ShellVerb::SkipBackward).unwrap();
        let expected = headings
            .iter()
            .filter(|heading| heading.position_secs < position)
            .next_back()
            .map_or_else(
                || {
                    if headings.is_empty() {
                        (position - 10.0).max(0.0)
                    } else {
                        0.0
                    }
                },
                |heading| heading.position_secs,
            );
        prop_assert!(
            (snapshot.position_secs - expected).abs() < 1e-9,
            "backward skip from {} with headings {:?} must land at {}, got {}",
            position,
            headings,
            expected,
            snapshot.position_secs,
        );
    }
}
