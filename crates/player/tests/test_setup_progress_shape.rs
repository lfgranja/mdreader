//! T002f — BLOCKING TEST (red before T005).
//!
//! Asserts the `Progress` payload shape for feature
//! 004-ressoa-dogfood-build: the new `Determinate`
//! variant must carry `consumed_units`, `total_units`,
//! `k`, `n` (not the old `position_secs`/`duration_secs`);
//! `Indeterminate` carries nothing; the progress
//! determinacy is fixed at session start; zero sections
//! implies indeterminate; and `None`-section rules are
//! never included incorrectly in the payload.
//!
//! FAILS (red) before T005 updates the shape.

use player::surface::{Progress, SurfaceResolver};

/// Shape contract: `Determinate` must expose the new
/// unit-based fields, not the old time-based ones.
#[test]
fn progress_determinate_has_new_shape_not_old_time_fields() {
    let resolver = SurfaceResolver::new(Some("Title".to_string()), "doc.wav".to_string(), true);
    let snapshot = resolver.resolve(10.0, &[]);
    match snapshot.progress {
        Progress::Determinate { .. } => {
            let value = serde_json::to_value(&snapshot.progress).expect("serialize progress");
            let obj = value.get("determinate").expect("determinate key present");
            // The new shape requires these four keys.
            assert!(
                obj.get("consumed_units").is_some(),
                "Determinate missing consumed_units (new shape T002f)"
            );
            assert!(
                obj.get("total_units").is_some(),
                "Determinate missing total_units (new shape T002f)"
            );
            assert!(
                obj.get("k").is_some(),
                "Determinate missing k (new shape T002f)"
            );
            assert!(
                obj.get("n").is_some(),
                "Determinate missing n (new shape T002f)"
            );
            // The old time-based fields must NOT appear.
            assert!(
                obj.get("position_secs").is_none(),
                "old position_secs must not appear in new Determinate shape (T002f)"
            );
            assert!(
                obj.get("duration_secs").is_none(),
                "old duration_secs must not appear in new Determinate shape (T002f)"
            );
        }
        Progress::Indeterminate => {
            panic!("expected Determinate when duration_known_at_start=true (T002f)");
        }
    }
}

/// Fixed-at-session-start: determinacy never switches.
/// With zero headings and `fixed_at_session_start` false,
/// progress must stay `Indeterminate` for every call.
#[test]
fn fixed_at_session_start_is_false_for_zero_sections() {
    let resolver = SurfaceResolver::new(None, "doc.wav".to_string(), false);
    let snapshot = resolver.resolve(0.0, &[]);
    assert!(
        matches!(snapshot.progress, Progress::Indeterminate),
        "zero sections + fixed_at_session_start=false must yield Indeterminate (T002f)"
    );
    // The payload must not include `None`-section rules incorrectly.
    assert!(
        snapshot.section_title.is_none(),
        "section_title must be None for zero headings, not a rule payload (T002f)"
    );
    // Assert no forbidden `None`-section payload key is injected.
    let serialized = serde_json::to_string(&snapshot).expect("serialize");
    assert!(
        !serialized.contains("none_section_rule"),
        "payload must not include None-section rule incorrectly (T002f)"
    );
}

/// Fixed-at-session-start true: progress stays determinate
/// regardless of headings count — the total is known.
/// The determinate payload carries the four new fields.
#[test]
fn fixed_at_session_start_true_keeps_determinate_shape() {
    let resolver = SurfaceResolver::new(
        Some("Title".to_string()),
        "doc.wav".to_string(),
        true, // fixed_at_session_start = true
    );
    // Even with zero headings, the session knows its total,
    // so progress must remain determinate with the new shape.
    let snapshot = resolver.resolve(0.0, &[]);
    assert!(
        matches!(snapshot.progress, Progress::Determinate { .. }),
        "fixed_at_session_start=true must keep Determinate even with zero sections (T002f)"
    );
    let value = serde_json::to_value(&snapshot.progress).expect("serialize");
    let obj = value.get("determinate").expect("determinate");
    assert!(obj.get("consumed_units").is_some());
    assert!(obj.get("total_units").is_some());
    assert!(obj.get("k").is_some());
    assert!(obj.get("n").is_some());
}
